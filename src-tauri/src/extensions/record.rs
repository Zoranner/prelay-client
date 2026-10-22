use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    agents::{AgentClient, AgentItem, AgentItemKind, AgentItemSource, AgentItemStatus},
    relay::client::ClientError,
};

use super::atomic_write;

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const EXTENSION_RECORD_FILE: &str = "extensions.json";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct InstalledExtensions(pub(crate) BTreeMap<String, InstalledExtensionRecord>);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledExtensionRecord {
    pub version: String,
    pub commit_sha: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub legacy: bool,
    #[serde(default)]
    pub targets: BTreeMap<String, InstalledTargetRecord>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledTargetRecord {
    #[serde(default)]
    pub rule: bool,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub mcps: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<String>,
}

pub(crate) fn upsert_extension_record(
    home: &Path,
    package: &str,
    version: &str,
    commit_sha: &str,
    targets: BTreeMap<String, InstalledTargetRecord>,
) -> Result<(), ClientError> {
    let mut installed = if extension_record_path(home).exists() {
        read_installed_extensions(home)?
    } else {
        legacy_records(home)
    };
    installed.0.insert(
        package.to_string(),
        InstalledExtensionRecord {
            version: version.to_string(),
            commit_sha: commit_sha.to_string(),
            legacy: false,
            targets,
        },
    );
    write_installed_extensions(home, &installed)
}

pub(crate) fn read_installed_extensions(home: &Path) -> Result<InstalledExtensions, ClientError> {
    let path = extension_record_path(home);
    match fs::read(&path) {
        Ok(contents) => serde_json::from_slice(&contents).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法读取扩展安装记录：{error}"),
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(InstalledExtensions::default())
        }
        Err(error) => Err(storage_error(error)),
    }
}

pub(crate) fn client_key(client: AgentClient) -> &'static str {
    match client {
        AgentClient::CodexCli => "codexCli",
        AgentClient::ChatGpt => "chatgpt",
        AgentClient::OpenCode => "openCode",
    }
}

pub(crate) fn managed_package_names(
    home: &Path,
    client: AgentClient,
) -> Result<std::collections::BTreeSet<String>, ClientError> {
    Ok(read_installed_extensions(home)?
        .0
        .into_iter()
        .filter(|(_, record)| {
            record
                .targets
                .get(client_key(client))
                .is_some_and(target_has_members)
        })
        .map(|(package, _)| package)
        .collect())
}

pub(crate) fn plugin_items(home: &Path, client: AgentClient) -> Vec<AgentItem> {
    let Ok(installed) = read_installed_extensions(home) else {
        return Vec::new();
    };
    let source_path = extension_record_path(home).display().to_string();
    installed
        .0
        .into_iter()
        .filter_map(|(package, record)| {
            let target = record.targets.get(client_key(client))?;
            let members = target_members(target);
            if members.is_empty() {
                return None;
            }
            Some(AgentItem {
                kind: AgentItemKind::Plugin,
                name: package.clone(),
                version: Some(record.version),
                package: Some(package),
                members,
                source: AgentItemSource::Team,
                source_path: source_path.clone(),
                status: AgentItemStatus::Enabled,
                error_message: None,
            })
        })
        .collect()
}

pub(crate) fn remove_extension_target(
    home: &Path,
    client: AgentClient,
    package: &str,
) -> Result<(), ClientError> {
    let mut installed = read_installed_extensions(home)?;
    let Some(record) = installed.0.get_mut(package) else {
        return Ok(());
    };
    record.targets.remove(client_key(client));
    if record.targets.is_empty() {
        installed.0.remove(package);
    }
    write_installed_extensions(home, &installed)
}

pub(crate) fn installed_target(
    home: &Path,
    client: AgentClient,
    package: &str,
) -> Result<Option<InstalledTargetRecord>, ClientError> {
    Ok(read_installed_extensions(home)?
        .0
        .get(package)
        .and_then(|record| record.targets.get(client_key(client)))
        .cloned())
}

pub(crate) fn write_installed_extensions(
    home: &Path,
    installed: &InstalledExtensions,
) -> Result<(), ClientError> {
    let contents = serde_json::to_vec_pretty(installed).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存扩展安装记录：{error}"),
        )
    })?;
    atomic_write(&extension_record_path(home), &contents)
}

pub(crate) fn target_has_members(target: &InstalledTargetRecord) -> bool {
    !target.skills.is_empty() || !target.mcps.is_empty() || !target.hooks.is_empty()
}

fn target_members(target: &InstalledTargetRecord) -> Vec<String> {
    let mut members = target
        .skills
        .iter()
        .map(|skill| format!("Skill · {skill}"))
        .collect::<Vec<_>>();
    members.extend(target.mcps.iter().map(|server| format!("MCP · {server}")));
    members.extend(target.hooks.iter().map(|hook| format!("Hook · {hook}")));
    members
}

fn legacy_records(home: &Path) -> InstalledExtensions {
    let mut installed = InstalledExtensions::default();
    merge_legacy_skills(home, &mut installed);
    merge_legacy_mcp(
        &home.join(".codex").join(".prelay").join("mcp.json"),
        &["codexCli", "chatgpt"],
        &mut installed,
    );
    merge_legacy_mcp(
        &home
            .join(".config")
            .join("opencode")
            .join(".prelay")
            .join("mcp.json"),
        &["openCode"],
        &mut installed,
    );
    merge_legacy_rule(
        &home.join(".codex").join(".prelay").join("rule.json"),
        &["codexCli", "chatgpt"],
        &mut installed,
    );
    merge_legacy_rule(
        &home
            .join(".config")
            .join("opencode")
            .join(".prelay")
            .join("rule.json"),
        &["openCode"],
        &mut installed,
    );
    installed
}

fn merge_legacy_skills(home: &Path, installed: &mut InstalledExtensions) {
    let Some(document) = read_legacy_json(&home.join(".agents").join(".prelay").join("skill.json"))
    else {
        return;
    };
    let Some(packages) = document.as_object() else {
        return;
    };
    for (package, value) in packages {
        let Some(value) = value.as_object() else {
            continue;
        };
        let version = string_field(value, "version");
        let commit_sha = string_field(value, "commitSha");
        let skills = value
            .get("skills")
            .and_then(Value::as_array)
            .map(|skills| {
                skills
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for client in ["codexCli", "chatgpt", "openCode"] {
            let target = ensure_legacy_record(installed, package, &version, &commit_sha, client);
            target.skills = skills.clone();
        }
    }
}

fn merge_legacy_mcp(path: &Path, clients: &[&str], installed: &mut InstalledExtensions) {
    let Some(document) = read_legacy_json(path) else {
        return;
    };
    let Some(packages) = document.as_object() else {
        return;
    };
    for (package, value) in packages {
        let Some(value) = value.as_object() else {
            continue;
        };
        let version = string_field(value, "version");
        let commit_sha = string_field(value, "commitSha");
        let server = string_field(value, "serverName");
        if server.is_empty() {
            continue;
        }
        for client in clients {
            let target = ensure_legacy_record(installed, package, &version, &commit_sha, client);
            target.mcps = vec![server.clone()];
        }
    }
}

fn merge_legacy_rule(path: &Path, clients: &[&str], installed: &mut InstalledExtensions) {
    let Some(document) = read_legacy_json(path) else {
        return;
    };
    let Some(value) = document.as_object() else {
        return;
    };
    let package = string_field(value, "package");
    if package.is_empty() {
        return;
    }
    let version = string_field(value, "version");
    let commit_sha = string_field(value, "commitSha");
    for client in clients {
        let target = ensure_legacy_record(installed, &package, &version, &commit_sha, client);
        target.rule = true;
    }
}

fn ensure_legacy_record<'a>(
    installed: &'a mut InstalledExtensions,
    package: &str,
    version: &str,
    commit_sha: &str,
    client: &str,
) -> &'a mut InstalledTargetRecord {
    let record =
        installed
            .0
            .entry(package.to_string())
            .or_insert_with(|| InstalledExtensionRecord {
                version: version.to_string(),
                commit_sha: commit_sha.to_string(),
                legacy: true,
                targets: BTreeMap::new(),
            });
    record.legacy = true;
    if record.version.is_empty() && !version.is_empty() {
        record.version = version.to_string();
    }
    if record.commit_sha.is_empty() && !commit_sha.is_empty() {
        record.commit_sha = commit_sha.to_string();
    }
    record.targets.entry(client.to_string()).or_default()
}

fn read_legacy_json(path: &Path) -> Option<Value> {
    let contents = fs::read(path).ok()?;
    serde_json::from_slice(&contents).ok()
}

fn string_field(value: &serde_json::Map<String, Value>, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn extension_record_path(home: &Path) -> PathBuf {
    home.join(PRELAY_STATE_DIRECTORY)
        .join(EXTENSION_RECORD_FILE)
}

fn storage_error(error: std::io::Error) -> ClientError {
    ClientError::new(
        "local_extensions_error",
        format!("无法读写扩展安装记录：{error}"),
    )
}

#[cfg(test)]
mod tests;
