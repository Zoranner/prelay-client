use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentClient {
    Codex,
    ClaudeCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentItemKind {
    Mcp,
    Skill,
    Plugin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentItemStatus {
    Enabled,
    Disabled,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItem {
    pub kind: AgentItemKind,
    pub name: String,
    pub version: Option<String>,
    pub source_path: String,
    pub status: AgentItemStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClientItems {
    pub client: AgentClient,
    pub items: Vec<AgentItem>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItemsSnapshot {
    pub clients: Vec<AgentClientItems>,
}

pub fn scan_user_items(home: &Path) -> AgentItemsSnapshot {
    let mut snapshot = AgentItemsSnapshot::default();
    add_client_items(&mut snapshot, AgentClient::Codex, scan_codex(home));
    add_client_items(
        &mut snapshot,
        AgentClient::ClaudeCode,
        scan_claude_code(home),
    );
    snapshot
}

fn add_client_items(snapshot: &mut AgentItemsSnapshot, client: AgentClient, items: Vec<AgentItem>) {
    if !items.is_empty() {
        snapshot.clients.push(AgentClientItems { client, items });
    }
}

fn scan_codex(home: &Path) -> Vec<AgentItem> {
    let codex_root = home.join(".codex");
    if !codex_root.exists() {
        return Vec::new();
    }
    let config_path = codex_root.join("config.toml");
    let mut items = match read_toml(&config_path) {
        Ok(Some(value)) => {
            let mut items = toml_items(&value, "mcp_servers", AgentItemKind::Mcp, &config_path);
            items.extend(codex_plugin_items(&value, &codex_root, &config_path));
            items
        }
        Ok(None) => Vec::new(),
        Err(()) => vec![error_item(AgentItemKind::Mcp, &config_path)],
    };
    items.extend(scan_skills(codex_root.join("skills")));
    items.extend(scan_skills(home.join(".agents").join("skills")));
    deduplicate(items)
}

fn scan_claude_code(home: &Path) -> Vec<AgentItem> {
    let claude_root = home.join(".claude");
    let config_path = home.join(".claude.json");
    if !claude_root.exists() && !config_path.exists() {
        return Vec::new();
    }
    let mut items = parse_claude_mcp_items(&config_path);
    items.extend(parse_claude_plugins(
        claude_root.join("plugins").join("installed_plugins.json"),
    ));
    items.extend(scan_skills(claude_root.join("skills")));
    deduplicate(items)
}

fn read_toml(path: &Path) -> Result<Option<toml::Value>, ()> {
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| toml::from_str::<toml::Value>(&contents).ok())
        .ok_or(())
        .map(Some)
}

fn toml_items(
    value: &toml::Value,
    section: &str,
    kind: AgentItemKind,
    path: &Path,
) -> Vec<AgentItem> {
    let Some(entries) = value.get(section).and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    entries
        .iter()
        .map(|(name, entry)| AgentItem {
            kind,
            name: name.to_owned(),
            version: None,
            source_path: path.display().to_string(),
            status: if entry
                .get("enabled")
                .and_then(toml::Value::as_bool)
                .is_some_and(|enabled| !enabled)
            {
                AgentItemStatus::Disabled
            } else {
                AgentItemStatus::Enabled
            },
            error_message: None,
        })
        .collect()
}

fn codex_plugin_items(
    value: &toml::Value,
    codex_root: &Path,
    config_path: &Path,
) -> Vec<AgentItem> {
    let Some(entries) = value.get("plugins").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    entries
        .iter()
        .map(|(name, entry)| {
            let Some(cache_path) = codex_plugin_cache_path(codex_root, name) else {
                return AgentItem {
                    kind: AgentItemKind::Plugin,
                    name: name.to_owned(),
                    version: None,
                    source_path: config_path.display().to_string(),
                    status: AgentItemStatus::Error,
                    error_message: Some("未找到已登记插件的本地缓存。".to_string()),
                };
            };
            AgentItem {
                kind: AgentItemKind::Plugin,
                name: name.to_owned(),
                version: None,
                source_path: cache_path.display().to_string(),
                status: if entry
                    .get("enabled")
                    .and_then(toml::Value::as_bool)
                    .is_some_and(|enabled| !enabled)
                {
                    AgentItemStatus::Disabled
                } else {
                    AgentItemStatus::Enabled
                },
                error_message: None,
            }
        })
        .collect()
}

fn codex_plugin_cache_path(codex_root: &Path, plugin_id: &str) -> Option<PathBuf> {
    let (plugin_name, marketplace) = plugin_id.rsplit_once('@')?;
    let cache_root = codex_root
        .join("plugins")
        .join("cache")
        .join(marketplace)
        .join(plugin_name);
    fs::read_dir(cache_root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join(".codex-plugin").join("plugin.json").is_file())
        .max_by_key(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .ok()
        })
}

fn parse_claude_mcp_items(path: &Path) -> Vec<AgentItem> {
    if !path.exists() {
        return Vec::new();
    }
    let value = match fs::read_to_string(path).and_then(|contents| {
        serde_json::from_str::<serde_json::Value>(&contents).map_err(std::io::Error::other)
    }) {
        Ok(value) => value,
        Err(_) => return vec![error_item(AgentItemKind::Mcp, path)],
    };
    value
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
        .map(|(name, _)| AgentItem {
            kind: AgentItemKind::Mcp,
            name: name.to_owned(),
            version: None,
            source_path: path.display().to_string(),
            status: AgentItemStatus::Enabled,
            error_message: None,
        })
        .collect()
}

fn parse_claude_plugins(path: PathBuf) -> Vec<AgentItem> {
    if !path.exists() {
        return Vec::new();
    }
    let value = match fs::read_to_string(&path).and_then(|contents| {
        serde_json::from_str::<serde_json::Value>(&contents).map_err(std::io::Error::other)
    }) {
        Ok(value) => value,
        Err(_) => return vec![error_item(AgentItemKind::Plugin, &path)],
    };
    value
        .get("plugins")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
        .flat_map(|(name, records)| {
            records
                .as_array()
                .into_iter()
                .flatten()
                .filter(|record| {
                    record.get("scope").and_then(serde_json::Value::as_str) == Some("user")
                })
                .map(|record| AgentItem {
                    kind: AgentItemKind::Plugin,
                    name: name.to_owned(),
                    version: record
                        .get("version")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned),
                    source_path: record
                        .get("installPath")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                        .unwrap_or_else(|| path.display().to_string()),
                    status: AgentItemStatus::Enabled,
                    error_message: None,
                })
        })
        .collect()
}

fn scan_skills(root: PathBuf) -> Vec<AgentItem> {
    let mut skills = Vec::new();
    visit_skill_directory(&root, &mut skills);
    skills
}

fn visit_skill_directory(path: &Path, skills: &mut Vec<AgentItem>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.is_file() {
                skills.push(AgentItem {
                    kind: AgentItemKind::Skill,
                    name: entry.file_name().to_string_lossy().to_string(),
                    version: None,
                    source_path: path.display().to_string(),
                    status: AgentItemStatus::Enabled,
                    error_message: None,
                });
            }
            visit_skill_directory(&path, skills);
        }
    }
}

fn error_item(kind: AgentItemKind, path: &Path) -> AgentItem {
    AgentItem {
        kind,
        name: "配置读取失败".to_string(),
        version: None,
        source_path: path.display().to_string(),
        status: AgentItemStatus::Error,
        error_message: Some("无法读取扩展配置。".to_string()),
    }
}

fn deduplicate(mut items: Vec<AgentItem>) -> Vec<AgentItem> {
    items.sort_by(|left, right| {
        (
            left.kind as u8,
            left.name.as_str(),
            left.source_path.as_str(),
        )
            .cmp(&(
                right.kind as u8,
                right.name.as_str(),
                right.source_path.as_str(),
            ))
    });
    items.dedup_by(|left, right| {
        left.kind == right.kind && left.name == right.name && left.source_path == right.source_path
    });
    items
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::tempdir;

    use super::{scan_user_items, AgentClient, AgentItemKind, AgentItemStatus};

    #[test]
    fn scans_user_level_codex_items_and_distinguishes_disabled_entries() {
        let directory = tempdir().unwrap();
        write(
            directory.path().join(".codex").join("config.toml"),
            r#"
[mcp_servers.research]
command = "prelay-search"

[mcp_servers.retired]
enabled = false
command = "retired-search"

[plugins."research@prelay"]
enabled = true
"#,
        );
        write(
            directory
                .path()
                .join(".codex")
                .join("plugins")
                .join("cache")
                .join("prelay")
                .join("research")
                .join("0.1.0")
                .join(".codex-plugin")
                .join("plugin.json"),
            "{}",
        );
        write(
            directory
                .path()
                .join(".agents")
                .join("skills")
                .join("web-research")
                .join("SKILL.md"),
            "---\nname: web-research\n---\n",
        );

        let snapshot = scan_user_items(directory.path());
        assert_eq!(snapshot.clients.len(), 1);
        let codex = snapshot
            .clients
            .iter()
            .find(|client| client.client == AgentClient::Codex)
            .unwrap();

        assert!(codex.items.iter().any(|item| {
            item.kind == AgentItemKind::Mcp
                && item.name == "research"
                && item.status == AgentItemStatus::Enabled
        }));
        assert!(codex.items.iter().any(|item| {
            item.kind == AgentItemKind::Mcp
                && item.name == "retired"
                && item.status == AgentItemStatus::Disabled
        }));
        assert!(codex.items.iter().any(|item| {
            item.kind == AgentItemKind::Plugin
                && item.name == "research@prelay"
                && item.status == AgentItemStatus::Enabled
                && item.source_path
                    == directory
                        .path()
                        .join(".codex")
                        .join("plugins")
                        .join("cache")
                        .join("prelay")
                        .join("research")
                        .join("0.1.0")
                        .display()
                        .to_string()
        }));
        assert!(codex.items.iter().any(|item| {
            item.kind == AgentItemKind::Skill
                && item.name == "web-research"
                && item.status == AgentItemStatus::Enabled
        }));
    }

    #[test]
    fn records_invalid_client_configuration_as_one_error() {
        let directory = tempdir().unwrap();
        write(
            directory.path().join(".codex").join("config.toml"),
            "[mcp_servers.invalid",
        );

        let snapshot = scan_user_items(directory.path());
        let codex = snapshot
            .clients
            .iter()
            .find(|client| client.client == AgentClient::Codex)
            .unwrap();

        assert_eq!(codex.items.len(), 1);
        assert_eq!(codex.items[0].status, AgentItemStatus::Error);
    }

    #[test]
    fn reports_a_configured_plugin_without_a_local_cache_as_an_error() {
        let directory = tempdir().unwrap();
        write(
            directory.path().join(".codex").join("config.toml"),
            "[plugins.\"search@prelay\"]\nenabled = true\n",
        );

        let snapshot = scan_user_items(directory.path());
        let plugin = snapshot
            .clients
            .iter()
            .find(|client| client.client == AgentClient::Codex)
            .and_then(|client| {
                client
                    .items
                    .iter()
                    .find(|item| item.kind == AgentItemKind::Plugin)
            })
            .unwrap();

        assert_eq!(plugin.status, AgentItemStatus::Error);
        assert_eq!(
            plugin.source_path,
            directory
                .path()
                .join(".codex")
                .join("config.toml")
                .display()
                .to_string()
        );
    }

    #[test]
    fn does_not_treat_agents_skills_as_claude_code_skills() {
        let directory = tempdir().unwrap();
        write(
            directory.path().join(".claude.json"),
            r#"{"mcpServers":{"prelay-search":{}}}"#,
        );
        write(
            directory
                .path()
                .join(".agents")
                .join("skills")
                .join("cavecrew")
                .join("SKILL.md"),
            "---\nname: cavecrew\n---\n",
        );

        let snapshot = scan_user_items(directory.path());
        let claude_code = snapshot
            .clients
            .iter()
            .find(|client| client.client == AgentClient::ClaudeCode)
            .unwrap();

        assert!(claude_code
            .items
            .iter()
            .all(|item| { item.kind != AgentItemKind::Skill || item.name != "cavecrew" }));
    }

    #[test]
    fn uses_native_separators_for_skill_source_paths() {
        let directory = tempdir().unwrap();
        write(
            directory.path().join(".codex").join("config.toml"),
            "[mcp_servers.research]\ncommand = \"prelay-search\"\n",
        );
        write(
            directory
                .path()
                .join(".agents")
                .join("skills")
                .join("cavecrew")
                .join("SKILL.md"),
            "---\nname: cavecrew\n---\n",
        );

        let snapshot = scan_user_items(directory.path());
        let skill = snapshot
            .clients
            .iter()
            .find(|client| client.client == AgentClient::Codex)
            .and_then(|client| {
                client
                    .items
                    .iter()
                    .find(|item| item.kind == AgentItemKind::Skill)
            })
            .unwrap();

        assert_eq!(
            skill.source_path,
            directory
                .path()
                .join(".agents")
                .join("skills")
                .join("cavecrew")
                .display()
                .to_string()
        );
    }

    #[test]
    fn omits_clients_without_user_level_artifacts() {
        let directory = tempdir().unwrap();

        assert!(scan_user_items(directory.path()).clients.is_empty());
    }

    fn write(path: impl AsRef<Path>, contents: &str) {
        let path = path.as_ref();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}
