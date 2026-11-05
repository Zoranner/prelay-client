use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use prelay_protocol::ExtensionMcpManifest;
use serde::Deserialize;

use crate::relay::client::ClientError;

use super::{
    hash_bytes, normalize_path, write_json, write_plugin, write_rules, ExtensionHost, HostState,
    PluginArtifact, PluginRecord, PluginState, RuleEntry, RuleState, PLUGIN_SCHEMA_VERSION,
    RULE_SCHEMA_VERSION,
};

mod backup;
use backup::backup_legacy;
mod read;
use read::{read_legacy_skill_roots, read_map, read_optional};

const MARKER: &str = ".migrated";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyRecord {
    #[serde(default)]
    version: String,
    #[serde(default)]
    commit_sha: String,
    #[serde(default)]
    targets: BTreeMap<String, LegacyTarget>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyTarget {
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    mcps: Vec<String>,
    #[serde(default)]
    hooks: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyHookState {
    #[serde(default)]
    version: String,
    #[serde(default)]
    commit_sha: String,
    #[serde(default)]
    hashes: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyMcpState {
    server_name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    commit_sha: String,
    #[serde(default)]
    manifest: Option<ExtensionMcpManifest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacySkillState {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    commit_sha: Option<String>,
    #[serde(default)]
    skills: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyRuleState {
    package: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    commit_sha: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacySkillRoot {
    package: String,
    #[serde(default)]
    roots: BTreeSet<String>,
}

struct PackageBuilder {
    version: String,
    commit_sha: String,
    conflict: bool,
    skills: BTreeSet<String>,
    mcps: BTreeMap<String, String>,
    hooks: BTreeMap<(String, Option<String>), String>,
}

impl PackageBuilder {
    fn new() -> Self {
        Self {
            version: String::new(),
            commit_sha: String::new(),
            conflict: false,
            skills: BTreeSet::new(),
            mcps: BTreeMap::new(),
            hooks: BTreeMap::new(),
        }
    }

    fn offer_version(&mut self, version: &str, commit_sha: &str) {
        if version.is_empty() && commit_sha.is_empty() {
            return;
        }
        if self.version.is_empty() && self.commit_sha.is_empty() {
            self.version = version.to_string();
            self.commit_sha = commit_sha.to_string();
            return;
        }
        if self.version != version || self.commit_sha != commit_sha {
            self.conflict = true;
        }
    }
}

pub(crate) fn ensure_migrated_locked(home: &Path) -> Result<(), ClientError> {
    let host = ExtensionHost::Codex;
    let marker = host.root(home).join(MARKER);
    if marker.exists() {
        return Ok(());
    }
    let plugins_dir = host.plugins_dir(home);
    let has_new_state = plugins_dir
        .read_dir()
        .map(|entries| {
            entries
                .flatten()
                .any(|entry| entry.path().extension().is_some())
        })
        .unwrap_or(false)
        || host.rule_path(home).exists();
    if has_new_state {
        write_marker(&marker)?;
        return Ok(());
    }
    let legacy = LegacyFiles::read(home)?;
    if legacy.is_empty() {
        write_marker(&marker)?;
        return Ok(());
    }

    let backup = backup_legacy(home)?;
    let plugins = build_plugins(home, &legacy)?;
    let rules = build_rules(home, &legacy)?;
    for plugin in &plugins {
        write_plugin(home, host, plugin)?;
    }
    write_rules(home, host, &rules)?;

    let loaded = HostState::load_locked(home, host)?;
    if loaded.plugins.len() != plugins.len() || loaded.rules.targets.len() != rules.targets.len() {
        return Err(ClientError::new(
            "local_extensions_error",
            format!("扩展状态迁移校验失败，旧状态保留在 {}。", backup.display()),
        ));
    }

    legacy.remove(home)?;
    write_marker(&marker)
}

fn write_marker(path: &Path) -> Result<(), ClientError> {
    write_json(path, &serde_json::json!({ "schemaVersion": 1 }))
}

fn build_plugins(home: &Path, legacy: &LegacyFiles) -> Result<Vec<PluginRecord>, ClientError> {
    let mut builders = BTreeMap::<String, PackageBuilder>::new();
    for (package, record) in &legacy.extensions {
        let builder = builders
            .entry(package.clone())
            .or_insert_with(PackageBuilder::new);
        builder.offer_version(&record.version, &record.commit_sha);
        for target in record.targets.values() {
            builder.skills.extend(target.skills.iter().cloned());
            for server in &target.mcps {
                builder.mcps.entry(server.clone()).or_default();
            }
            if !target.hooks.is_empty() && !legacy.hooks.contains_key(package) {
                builder.conflict = true;
            }
        }
    }
    for (package, entry) in &legacy.hooks {
        let builder = builders
            .entry(package.clone())
            .or_insert_with(PackageBuilder::new);
        builder.offer_version(&entry.version, &entry.commit_sha);
        for hash in &entry.hashes {
            if let Some(handler) = legacy.handlers.get(hash) {
                builder.hooks.insert(
                    (handler.event.clone(), handler.matcher.clone()),
                    hash.clone(),
                );
            } else {
                builder.conflict = true;
            }
        }
    }
    for (package, entry) in &legacy.mcps {
        let builder = builders
            .entry(package.clone())
            .or_insert_with(PackageBuilder::new);
        builder.offer_version(&entry.version, &entry.commit_sha);
        let hash = match entry.manifest.as_ref() {
            Some(manifest) => crate::extensions::mcp::manifest_fingerprint(home, manifest)
                .unwrap_or_else(|_| hash_bytes(b"")),
            None => hash_bytes(b""),
        };
        builder.mcps.insert(entry.server_name.clone(), hash);
    }
    for (package, entry) in &legacy.skills {
        let builder = builders
            .entry(package.clone())
            .or_insert_with(PackageBuilder::new);
        builder.offer_version(
            entry.version.as_deref().unwrap_or_default(),
            entry.commit_sha.as_deref().unwrap_or_default(),
        );
        builder.skills.extend(entry.skills.iter().cloned());
    }

    let skill_root = home.join(".agents").join("skills");
    let mcp_target = home.join(".codex").join("config.toml");
    let hook_target = home.join(".codex").join("hooks.json");
    let mut plugins = Vec::new();
    for (package, builder) in builders {
        let mut artifacts = Vec::new();
        if !builder.skills.is_empty() {
            artifacts.push(PluginArtifact::Skill {
                target: normalize_path(&skill_root),
                names: builder.skills.into_iter().collect(),
            });
        }
        for (server_name, manifest_hash) in builder.mcps {
            artifacts.push(PluginArtifact::Mcp {
                target: normalize_path(&mcp_target),
                server_name,
                manifest_hash,
            });
        }
        for ((event, matcher), handler_hash) in builder.hooks {
            artifacts.push(PluginArtifact::Hook {
                target: normalize_path(&hook_target),
                event,
                matcher,
                handler_hash,
            });
        }
        if artifacts.is_empty() {
            continue;
        }
        plugins.push(PluginRecord {
            schema_version: PLUGIN_SCHEMA_VERSION,
            name: package,
            version: builder.version,
            commit_sha: builder.commit_sha,
            state: if builder.conflict {
                PluginState::Installing
            } else {
                PluginState::Installed
            },
            artifacts,
        });
    }
    Ok(plugins)
}

fn build_rules(home: &Path, legacy: &LegacyFiles) -> Result<RuleState, ClientError> {
    let mut rules = RuleState {
        schema_version: RULE_SCHEMA_VERSION,
        targets: BTreeMap::new(),
    };
    let Some(entry) = legacy.rule.as_ref() else {
        return Ok(rules);
    };
    if entry.package.is_empty() {
        return Ok(rules);
    }
    let target = home.join(".codex").join("AGENTS.md");
    let content_hash = fs::read(&target)
        .map(|contents| hash_bytes(&contents))
        .unwrap_or_else(|_| hash_bytes(b""));
    rules.targets.insert(
        normalize_path(&target),
        RuleEntry {
            package: entry.package.clone(),
            version: entry.version.clone(),
            commit_sha: entry.commit_sha.clone(),
            content_hash,
            state: PluginState::Installed,
        },
    );
    Ok(rules)
}

#[derive(Default)]
struct LegacyFiles {
    extensions: BTreeMap<String, LegacyRecord>,
    hooks: BTreeMap<String, LegacyHookState>,
    mcps: BTreeMap<String, LegacyMcpState>,
    skills: BTreeMap<String, LegacySkillState>,
    rule: Option<LegacyRuleState>,
    handlers: BTreeMap<String, crate::extensions::hooks::HookHandler>,
}

impl LegacyFiles {
    fn read(home: &Path) -> Result<Self, ClientError> {
        let handlers = crate::extensions::hooks::handlers(home)
            .unwrap_or_default()
            .into_iter()
            .map(|handler| (handler.hash.clone(), handler))
            .collect();
        let mut files = Self {
            handlers,
            ..Self::default()
        };
        files.extensions = read_map(&home.join(".prelay").join("extensions.json"))?;
        files.hooks = read_map(&home.join(".codex").join(".prelay").join("hooks.json"))?;
        files.mcps = read_map(&home.join(".codex").join(".prelay").join("mcp.json"))?;
        files.rule = read_optional(&home.join(".codex").join(".prelay").join("rule.json"))?;
        files.skills = read_map(&home.join(".agents").join(".prelay").join("skill.json"))?;
        for (package, roots) in
            read_legacy_skill_roots(&home.join(".agents").join(".prelay").join("skills"))?
        {
            let entry = files.skills.entry(package).or_insert(LegacySkillState {
                version: None,
                commit_sha: None,
                skills: BTreeSet::new(),
            });
            entry.skills.extend(roots);
        }
        Ok(files)
    }

    fn is_empty(&self) -> bool {
        self.extensions.is_empty()
            && self.hooks.is_empty()
            && self.mcps.is_empty()
            && self.skills.is_empty()
            && self.rule.is_none()
    }

    fn remove(&self, home: &Path) -> Result<(), ClientError> {
        let files = [
            home.join(".prelay").join("extensions.json"),
            home.join(".codex").join(".prelay").join("hooks.json"),
            home.join(".codex").join(".prelay").join("mcp.json"),
            home.join(".codex").join(".prelay").join("rule.json"),
            home.join(".agents").join(".prelay").join("skill.json"),
        ];
        for path in files {
            match fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(ClientError::new(
                        "local_extensions_error",
                        format!("无法删除旧扩展状态：{error}"),
                    ));
                }
            }
        }
        let legacy_skills = home.join(".agents").join(".prelay").join("skills");
        if legacy_skills.exists() {
            fs::remove_dir_all(&legacy_skills).map_err(|error| {
                ClientError::new(
                    "local_extensions_error",
                    format!("无法删除旧 Skill 状态：{error}"),
                )
            })?;
        }
        Ok(())
    }
}
