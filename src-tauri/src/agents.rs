use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use toml_edit::DocumentMut;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentClient {
    Codex,
    ClaudeCode,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
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

pub fn uninstall_user_item(
    home: &Path,
    client: AgentClient,
    kind: AgentItemKind,
    name: &str,
    source_path: &str,
) -> Result<(), String> {
    let item = scan_user_items(home)
        .clients
        .into_iter()
        .find(|items| items.client == client)
        .and_then(|items| {
            items.items.into_iter().find(|item| {
                item.kind == kind && item.name == name && item.source_path == source_path
            })
        })
        .ok_or_else(|| "未找到要卸载的本地条目。".to_string())?;

    if item.status == AgentItemStatus::Error {
        return Err("无法卸载配置读取失败的条目。".to_string());
    }

    match (client, kind) {
        (AgentClient::Codex, AgentItemKind::Mcp) => {
            remove_codex_config_item(home, "mcp_servers", name)
        }
        (AgentClient::ClaudeCode, AgentItemKind::Mcp) => remove_claude_mcp_item(home, name),
        (AgentClient::Codex, AgentItemKind::Plugin) => remove_codex_plugin(home, name),
        (AgentClient::ClaudeCode, AgentItemKind::Plugin) => remove_claude_plugin(home, name),
        (_, AgentItemKind::Skill) => remove_skill_directory(&item.source_path),
    }
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

fn remove_codex_config_item(home: &Path, section: &str, name: &str) -> Result<(), String> {
    let config_path = home.join(".codex").join("config.toml");
    let contents = fs::read_to_string(&config_path)
        .map_err(|error| format!("无法读取 Codex 配置：{error}"))?;
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    let table = document[section]
        .as_table_mut()
        .ok_or_else(|| "未找到要卸载的配置项。".to_string())?;
    if table.remove(name).is_none() {
        return Err("未找到要卸载的配置项。".to_string());
    }
    write_text(&config_path, document.to_string().as_bytes())
}

fn remove_codex_plugin(home: &Path, name: &str) -> Result<(), String> {
    remove_codex_config_item(home, "plugins", name)?;
    let (plugin_name, marketplace) = name
        .rsplit_once('@')
        .ok_or_else(|| "插件标识格式不正确。".to_string())?;
    remove_directory(
        &home
            .join(".codex")
            .join("plugins")
            .join("cache")
            .join(marketplace)
            .join(plugin_name),
    )
}

fn remove_claude_mcp_item(home: &Path, name: &str) -> Result<(), String> {
    let path = home.join(".claude.json");
    let mut document = read_claude_document(&path)?;
    let servers = document
        .get_mut("mcpServers")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| "未找到要卸载的 MCP 配置。".to_string())?;
    if servers.remove(name).is_none() {
        return Err("未找到要卸载的 MCP 配置。".to_string());
    }
    write_json(&path, &document)
}

fn remove_claude_plugin(home: &Path, name: &str) -> Result<(), String> {
    let path = home
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json");
    let mut document = read_claude_document(&path)?;
    let plugins = document
        .get_mut("plugins")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| "未找到要卸载的插件登记。".to_string())?;
    let records = plugins
        .get_mut(name)
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| "未找到要卸载的插件登记。".to_string())?;
    let install_paths = records
        .iter()
        .filter(|record| record.get("scope").and_then(serde_json::Value::as_str) == Some("user"))
        .filter_map(|record| {
            record
                .get("installPath")
                .and_then(serde_json::Value::as_str)
        })
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    records
        .retain(|record| record.get("scope").and_then(serde_json::Value::as_str) != Some("user"));
    if records.is_empty() {
        plugins.remove(name);
    }
    write_json(&path, &document)?;
    for install_path in install_paths {
        remove_directory(&install_path)?;
    }
    Ok(())
}

fn remove_skill_directory(source_path: &str) -> Result<(), String> {
    remove_directory(Path::new(source_path))
}

fn read_claude_document(path: &Path) -> Result<serde_json::Value, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("无法读取 Claude Code 配置：{error}"))
        .and_then(|contents| {
            serde_json::from_str(&contents)
                .map_err(|error| format!("Claude Code 配置不是有效的 JSON：{error}"))
        })
}

fn write_json(path: &Path, document: &serde_json::Value) -> Result<(), String> {
    let contents = serde_json::to_vec_pretty(document)
        .map_err(|error| format!("无法写入 Claude Code 配置：{error}"))?;
    write_text(path, &contents)
}

fn remove_directory(path: &Path) -> Result<(), String> {
    fs::remove_dir_all(path).map_err(|error| format!("无法删除本地文件：{error}"))
}

fn write_text(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("无法创建配置目录：{error}"))?;
    let mut file =
        AtomicWriteFile::open(path).map_err(|error| format!("无法打开配置文件：{error}"))?;
    file.write_all(contents)
        .map_err(|error| format!("无法写入配置文件：{error}"))?;
    file.sync_all()
        .map_err(|error| format!("无法同步配置文件：{error}"))?;
    file.commit()
        .map_err(|error| format!("无法保存配置文件：{error}"))
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

    use super::{
        scan_user_items, uninstall_user_item, AgentClient, AgentItemKind, AgentItemStatus,
    };

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
    fn uninstalls_codex_items_without_removing_other_entries() {
        let directory = tempdir().unwrap();
        let config_path = directory.path().join(".codex").join("config.toml");
        write(
            &config_path,
            r#"
[mcp_servers.keep]
command = "keep"

[mcp_servers.remove]
command = "remove"

[plugins."remove@prelay"]
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
                .join("remove")
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
                .join("remove")
                .join("SKILL.md"),
            "---\nname: remove\n---\n",
        );

        let snapshot = scan_user_items(directory.path());
        let codex = snapshot
            .clients
            .iter()
            .find(|client| client.client == AgentClient::Codex)
            .unwrap();
        for kind in [
            AgentItemKind::Mcp,
            AgentItemKind::Plugin,
            AgentItemKind::Skill,
        ] {
            let item = codex
                .items
                .iter()
                .find(|item| item.kind == kind && item.name == "remove")
                .or_else(|| {
                    codex.items.iter().find(|item| {
                        item.kind == AgentItemKind::Plugin && item.name == "remove@prelay"
                    })
                })
                .unwrap();
            uninstall_user_item(
                directory.path(),
                AgentClient::Codex,
                item.kind,
                &item.name,
                &item.source_path,
            )
            .unwrap();
        }

        let items = scan_user_items(directory.path()).clients.remove(0).items;
        assert!(items.iter().any(|item| item.name == "keep"));
        assert!(items.iter().all(|item| item.name != "remove"));
        assert!(items.iter().all(|item| item.name != "remove@prelay"));
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
