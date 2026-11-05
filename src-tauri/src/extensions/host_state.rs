use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{agents::AgentClient, relay::client::ClientError};

use super::atomic_write;

mod conflicts;
mod migrate;
mod support;
pub(crate) use migrate::ensure_migrated_locked;
pub(crate) use support::{hash_bytes, hash_json, lock, normalize_path, valid_plugin_name};

pub(crate) const PLUGIN_SCHEMA_VERSION: u32 = 2;
pub(crate) const RULE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ExtensionHost {
    Codex,
}

impl ExtensionHost {
    pub(crate) const ALL: [Self; 1] = [Self::Codex];

    pub(crate) fn from_client(client: AgentClient) -> Self {
        match client {
            AgentClient::CodexCli | AgentClient::ChatGpt => Self::Codex,
        }
    }

    /// 所有入口必须落在同一个宿主；空客户端或跨宿主时返回 None。
    pub(crate) fn from_clients(clients: &[AgentClient]) -> Option<Self> {
        let mut hosts = clients.iter().map(|client| Self::from_client(*client));
        let first = hosts.next()?;
        hosts.all(|host| host == first).then_some(first)
    }

    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Codex => "codex",
        }
    }

    pub(crate) fn root(self, home: &Path) -> PathBuf {
        home.join(".prelay").join("hosts").join(self.key())
    }

    pub(crate) fn plugins_dir(self, home: &Path) -> PathBuf {
        self.root(home).join("plugins")
    }

    pub(crate) fn plugin_path(self, home: &Path, name: &str) -> PathBuf {
        self.plugins_dir(home).join(format!("{name}.json"))
    }

    pub(crate) fn rule_path(self, home: &Path) -> PathBuf {
        self.root(home).join("rule.json")
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PluginState {
    Installing,
    Installed,
    Removing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArtifactHealth {
    Managed,
    Missing,
    Modified,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PluginRecord {
    pub schema_version: u32,
    pub name: String,
    pub version: String,
    pub commit_sha: String,
    pub state: PluginState,
    pub artifacts: Vec<PluginArtifact>,
}

impl PluginRecord {
    pub(crate) fn members(&self) -> Vec<String> {
        let mut members = Vec::new();
        if self
            .artifacts
            .iter()
            .any(|artifact| matches!(artifact, PluginArtifact::Skill { .. }))
        {
            members.push("Skill".to_string());
        }
        if self
            .artifacts
            .iter()
            .any(|artifact| matches!(artifact, PluginArtifact::Mcp { .. }))
        {
            members.push("MCP".to_string());
        }
        if self
            .artifacts
            .iter()
            .any(|artifact| matches!(artifact, PluginArtifact::Hook { .. }))
        {
            members.push("Hook".to_string());
        }
        members
    }

    pub(crate) fn hooks(&self) -> impl Iterator<Item = &PluginArtifact> {
        self.artifacts
            .iter()
            .filter(|artifact| matches!(artifact, PluginArtifact::Hook { .. }))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum PluginArtifact {
    Skill {
        target: String,
        names: Vec<String>,
    },
    Mcp {
        target: String,
        server_name: String,
        manifest_hash: String,
    },
    Hook {
        target: String,
        event: String,
        matcher: Option<String>,
        handler_hash: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuleEntry {
    pub package: String,
    pub version: String,
    pub commit_sha: String,
    pub content_hash: String,
    pub state: PluginState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuleState {
    pub schema_version: u32,
    pub targets: BTreeMap<String, RuleEntry>,
}

impl Default for RuleState {
    fn default() -> Self {
        Self {
            schema_version: RULE_SCHEMA_VERSION,
            targets: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StateError {
    pub path: String,
    pub message: String,
    pub readonly: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct HostState {
    pub host: ExtensionHost,
    pub plugins: Vec<PluginRecord>,
    pub rules: RuleState,
    pub errors: Vec<StateError>,
    pub readonly: bool,
    conflicts: BTreeSet<String>,
}

impl HostState {
    pub(crate) fn load(home: &Path, host: ExtensionHost) -> Result<Self, ClientError> {
        let _lock = lock(home, host)?;
        ensure_migrated_locked(home)?;
        Self::load_locked(home, host)
    }

    pub(crate) fn load_locked(home: &Path, host: ExtensionHost) -> Result<Self, ClientError> {
        let mut state = Self {
            host,
            plugins: Vec::new(),
            rules: RuleState::default(),
            errors: Vec::new(),
            readonly: false,
            conflicts: BTreeSet::new(),
        };
        state.load_plugins(home);
        state.load_rules(home);
        Ok(state)
    }

    pub(crate) fn plugin(&self, name: &str) -> Option<&PluginRecord> {
        self.plugins
            .iter()
            .find(|plugin| plugin.name == name && !self.conflicts.contains(&plugin.name))
    }

    pub(crate) fn active_plugins(&self) -> impl Iterator<Item = &PluginRecord> {
        self.plugins.iter().filter(|plugin| {
            plugin.state == PluginState::Installed && !self.conflicts.contains(&plugin.name)
        })
    }

    pub(crate) fn is_conflicted(&self, name: &str) -> bool {
        self.conflicts.contains(name)
    }

    pub(crate) fn plugin_names(&self) -> BTreeSet<String> {
        self.plugins
            .iter()
            .filter(|plugin| !self.conflicts.contains(&plugin.name))
            .map(|plugin| plugin.name.clone())
            .collect()
    }

    pub(crate) fn ensure_writable(&self) -> Result<(), ClientError> {
        if self.readonly || self.errors.iter().any(|error| error.readonly) {
            return Err(ClientError::new(
                "local_extensions_readonly",
                "扩展状态版本高于当前客户端支持范围，已进入只读模式。",
            ));
        }
        if !self.conflicts.is_empty() {
            return Err(ClientError::new(
                "local_extensions_conflict",
                "扩展制品由多个插件重复声明，请先处理冲突。",
            ));
        }
        Ok(())
    }

    pub(crate) fn rule_entry(&self, target: &Path) -> Option<&RuleEntry> {
        self.rules.targets.get(&normalize_path(target))
    }

    fn load_plugins(&mut self, home: &Path) {
        let directory = self.host.plugins_dir(home);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => {
                self.push_error(&directory, format!("无法读取插件状态目录：{error}"), false);
                return;
            }
        };
        let mut paths = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            match read_plugin_file(&path) {
                Ok(plugin) => self.plugins.push(plugin),
                Err(error) => {
                    let readonly = error.code() == "local_extensions_readonly";
                    self.push_error(&path, error.message, readonly);
                }
            }
        }
        let (conflicts, errors) = conflicts::detect_conflicts(&self.plugins, &directory);
        self.conflicts.extend(conflicts);
        self.errors.extend(errors);
    }

    fn load_rules(&mut self, home: &Path) {
        let path = self.host.rule_path(home);
        let contents = match fs::read(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => {
                self.push_error(&path, format!("无法读取规则状态：{error}"), false);
                return;
            }
        };
        let rules: RuleState = match serde_json::from_slice(&contents) {
            Ok(rules) => rules,
            Err(error) => {
                self.push_error(&path, format!("规则状态不是有效的 JSON：{error}"), false);
                return;
            }
        };
        if rules.schema_version > RULE_SCHEMA_VERSION {
            self.push_error(
                &path,
                "规则状态版本高于当前客户端支持范围。".to_string(),
                true,
            );
            return;
        }
        self.rules = rules;
    }

    fn push_error(&mut self, path: &Path, message: String, readonly: bool) {
        self.readonly |= readonly;
        self.errors.push(StateError {
            path: path.display().to_string(),
            message,
            readonly,
        });
    }
}

pub(crate) fn write_plugin(
    home: &Path,
    host: ExtensionHost,
    plugin: &PluginRecord,
) -> Result<(), ClientError> {
    if !valid_plugin_name(&plugin.name) {
        return Err(ClientError::new(
            "local_extensions_error",
            "插件名不能用作状态文件名。",
        ));
    }
    write_json(&host.plugin_path(home, &plugin.name), plugin)
}

pub(crate) fn remove_plugin(
    home: &Path,
    host: ExtensionHost,
    name: &str,
) -> Result<(), ClientError> {
    match fs::remove_file(host.plugin_path(home, name)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ClientError::new(
            "local_extensions_error",
            format!("无法删除插件状态：{error}"),
        )),
    }
}

pub(crate) fn write_rules(
    home: &Path,
    host: ExtensionHost,
    rules: &RuleState,
) -> Result<(), ClientError> {
    write_json(&host.rule_path(home), rules)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ClientError> {
    let contents = serde_json::to_vec_pretty(value).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法序列化扩展状态：{error}"),
        )
    })?;
    atomic_write(path, &contents)
}

fn read_plugin_file(path: &Path) -> Result<PluginRecord, ClientError> {
    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ClientError::new("not_found", "插件状态不存在。"));
        }
        Err(error) => {
            return Err(ClientError::new(
                "local_extensions_error",
                format!("无法读取插件状态：{error}"),
            ));
        }
    };
    let plugin: PluginRecord = serde_json::from_slice(&contents).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("插件状态不是有效的 JSON：{error}"),
        )
    })?;
    let file_name = path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    if plugin.name != file_name {
        return Err(ClientError::new(
            "local_extensions_error",
            "插件状态的文件名与 name 不一致。",
        ));
    }
    if plugin.schema_version > PLUGIN_SCHEMA_VERSION {
        return Err(ClientError::new(
            "local_extensions_readonly",
            "插件状态版本高于当前客户端支持范围。",
        ));
    }
    Ok(plugin)
}

#[cfg(test)]
mod tests;
