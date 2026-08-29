use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use prelay_protocol::{ExtensionMcpManifest, ExtensionMcpTransport};
use toml_edit::{value, Array, DocumentMut, Item, Table};

use super::{
    discovery::agent_client_is_installed,
    integrations,
    integrations::integration,
    model::{
        AgentClient, AgentClientItems, AgentItem, AgentItemKind, AgentItemStatus,
        AgentItemsSnapshot, REGISTERED_AGENT_CLIENTS,
    },
};

pub fn scan_user_items(home: &Path) -> AgentItemsSnapshot {
    scan_user_items_with_installation(home, agent_client_is_installed)
}

pub fn scan_agent_items(home: &Path, client: AgentClient) -> AgentClientItems {
    AgentClientItems {
        client,
        version: None,
        items: integration(client).scan(home),
    }
}

pub fn agent_rule_targets(clients: &[AgentClient], home: &Path) -> Vec<PathBuf> {
    clients
        .iter()
        .filter_map(|client| integration(*client).rule_target(home))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn agent_skill_target_roots(clients: &[AgentClient], home: &Path) -> Vec<PathBuf> {
    clients
        .iter()
        .filter_map(|client| integration(*client).skill_target_root(home))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn opencode_configuration_path(home: &Path) -> PathBuf {
    integrations::opencode::configuration_path(home)
}

pub(crate) fn scan_user_items_with_installation(
    home: &Path,
    is_installed: impl Fn(AgentClient) -> bool,
) -> AgentItemsSnapshot {
    let mut snapshot = AgentItemsSnapshot::default();
    for client in REGISTERED_AGENT_CLIENTS {
        if is_installed(client) {
            let items = integration(client).scan(home);
            snapshot.clients.push(AgentClientItems {
                client,
                version: None,
                items,
            });
        }
    }
    snapshot
}

pub fn uninstall_user_item(
    home: &Path,
    client: AgentClient,
    kind: AgentItemKind,
    name: &str,
    source_path: &str,
) -> Result<(), String> {
    uninstall_user_item_with_installation(
        home,
        client,
        kind,
        name,
        source_path,
        agent_client_is_installed,
    )
}

pub(crate) fn uninstall_user_item_with_installation(
    home: &Path,
    client: AgentClient,
    kind: AgentItemKind,
    name: &str,
    source_path: &str,
    is_installed: impl Fn(AgentClient) -> bool,
) -> Result<(), String> {
    let item = scan_user_items_with_installation(home, is_installed)
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

    integration(client).uninstall(home, kind, name, &item.source_path)
}

pub(crate) fn scan_codex(home: &Path) -> Vec<AgentItem> {
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

pub(crate) fn remove_codex_config_item(
    home: &Path,
    section: &str,
    name: &str,
) -> Result<(), String> {
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

pub(crate) fn upsert_codex_mcp_server(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<(), String> {
    let config_path = home.join(".codex").join("config.toml");
    let contents = fs::read_to_string(&config_path).unwrap_or_default();
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    let mut server = Table::new();
    match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            enabled,
            timeout_ms,
        } => {
            server["command"] = value(&command[0]);
            if command.len() > 1 {
                server["args"] = value(toml_array(&command[1..]));
            }
            if let Some(cwd) = cwd {
                server["cwd"] = value(cwd);
            }
            if !environment.is_empty() {
                server["env"] = Item::Table(toml_table(environment));
            }
            server["enabled"] = value(*enabled);
            if let Some(timeout_ms) = timeout_ms {
                server["startup_timeout_sec"] = value(timeout_seconds(*timeout_ms));
            }
        }
        ExtensionMcpTransport::Http {
            url,
            headers,
            enabled,
            timeout_ms,
        } => {
            server["url"] = value(url);
            if !headers.is_empty() {
                server["http_headers"] = Item::Table(toml_table(headers));
            }
            server["enabled"] = value(*enabled);
            if let Some(timeout_ms) = timeout_ms {
                server["startup_timeout_sec"] = value(timeout_seconds(*timeout_ms));
            }
        }
    }
    if document["mcp_servers"].is_none() {
        document["mcp_servers"] = Item::Table(Table::new());
    }
    document["mcp_servers"][&manifest.name] = Item::Table(server);
    write_text(&config_path, document.to_string().as_bytes())
}

pub(crate) fn register_codex_plugin(home: &Path, plugin_id: &str) -> Result<(), String> {
    let config_path = home.join(".codex").join("config.toml");
    let contents = fs::read_to_string(&config_path).unwrap_or_default();
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    if document["plugins"].is_none() {
        document["plugins"] = Item::Table(Table::new());
    }
    document["plugins"][plugin_id]["enabled"] = value(true);
    write_text(&config_path, document.to_string().as_bytes())
}

fn toml_array(values: &[String]) -> Array {
    let mut array = Array::new();
    for value in values {
        array.push(value.as_str());
    }
    array
}

fn timeout_seconds(timeout_ms: u64) -> i64 {
    timeout_ms.div_ceil(1_000).min(i64::MAX as u64) as i64
}

fn toml_table(values: &std::collections::BTreeMap<String, String>) -> Table {
    let mut table = Table::new();
    for (name, entry) in values {
        table[name] = value(entry);
    }
    table
}

pub(crate) fn remove_codex_plugin(home: &Path, name: &str) -> Result<(), String> {
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

pub(crate) fn remove_skill_directory(source_path: &str) -> Result<(), String> {
    remove_directory(Path::new(source_path))
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

pub(crate) fn write_json(path: &Path, document: &serde_json::Value) -> Result<(), String> {
    let contents = serde_json::to_vec_pretty(document)
        .map_err(|error| format!("无法序列化 JSON 配置：{error}"))?;
    write_text(path, &contents)
}

pub(crate) fn scan_skills(root: PathBuf) -> Vec<AgentItem> {
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

pub(crate) fn error_item(kind: AgentItemKind, path: &Path) -> AgentItem {
    AgentItem {
        kind,
        name: "配置读取失败".to_string(),
        version: None,
        source_path: path.display().to_string(),
        status: AgentItemStatus::Error,
        error_message: Some("无法读取扩展配置。".to_string()),
    }
}

pub(crate) fn deduplicate(mut items: Vec<AgentItem>) -> Vec<AgentItem> {
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
