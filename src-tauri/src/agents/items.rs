use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use prelay_protocol::{ExtensionMcpManifest, ExtensionMcpTransport};
use toml_edit::{value, Array, DocumentMut, Item, Table, Value};

use super::{
    discovery::agent_client_is_installed,
    integrations,
    integrations::integration,
    model::{
        AgentClient, AgentClientItems, AgentItem, AgentItemKind, AgentItemSource, AgentItemStatus,
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
    agent_rule_targets_with_clients(clients, home)
        .into_iter()
        .map(|(_, target)| target)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn agent_rule_targets_with_clients(
    clients: &[AgentClient],
    home: &Path,
) -> Vec<(AgentClient, PathBuf)> {
    clients
        .iter()
        .filter_map(|client| {
            integration(*client)
                .rule_target(home)
                .map(|target| (*client, target))
        })
        .collect()
}

pub fn agent_skill_target_roots(clients: &[AgentClient], home: &Path) -> Vec<PathBuf> {
    agent_skill_targets(clients, home)
        .into_iter()
        .map(|(_, root)| root)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn agent_skill_targets(clients: &[AgentClient], home: &Path) -> Vec<(AgentClient, PathBuf)> {
    clients
        .iter()
        .filter_map(|client| {
            integration(*client)
                .skill_target_root(home)
                .map(|root| (*client, root))
        })
        .collect()
}

pub(crate) fn opencode_configuration_path(home: &Path) -> PathBuf {
    integrations::opencode::configuration_path(home)
}

pub(crate) fn claude_code_configuration_path(home: &Path) -> PathBuf {
    integrations::claude_code::configuration_path(home)
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

    integration(client).uninstall(home, kind, name, &item.source_path)
}

pub(crate) fn scan_codex(home: &Path) -> Vec<AgentItem> {
    let codex_root = home.join(".codex");
    if !codex_root.exists() {
        return Vec::new();
    }
    let config_path = codex_root.join("config.toml");
    let mut items = match read_toml(&config_path) {
        Ok(Some(value)) => toml_items(&value, "mcp_servers", AgentItemKind::Mcp, &config_path),
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
            source: AgentItemSource::Personal,
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

pub(crate) fn codex_mcp_server_exists(home: &Path, name: &str) -> Result<bool, String> {
    let config_path = home.join(".codex").join("config.toml");
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("无法读取 Codex 配置：{error}")),
    };
    let document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    Ok(document["mcp_servers"]
        .as_table()
        .is_some_and(|servers| servers.contains_key(name)))
}

pub(crate) fn upsert_codex_mcp_server(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<(), String> {
    let config_path = home.join(".codex").join("config.toml");
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("无法读取 Codex 配置：{error}")),
    };
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    if !document.as_table().contains_key("mcp_servers") {
        document["mcp_servers"] = Item::Table(Table::new());
    }
    let servers = document["mcp_servers"]
        .as_table_mut()
        .ok_or_else(|| "Codex MCP 配置不是表。".to_string())?;
    servers.insert(
        &manifest.name,
        Item::Table(codex_mcp_server_table(manifest)?),
    );
    write_text(&config_path, document.to_string().as_bytes())
}

fn codex_mcp_server_table(manifest: &ExtensionMcpManifest) -> Result<Table, String> {
    let mut server = Table::new();
    match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            enabled,
            timeout_ms,
        } => {
            let (program, arguments) = command
                .split_first()
                .filter(|(program, _)| !program.trim().is_empty())
                .ok_or_else(|| "MCP 命令不能为空。".to_string())?;
            if cwd.is_some() {
                return Err("MCP 工作目录当前不受支持。".to_string());
            }
            if environment.iter().any(|(name, value)| name != value) {
                return Err("MCP 环境变量必须使用同名引用。".to_string());
            }
            server["command"] = value(program);
            set_string_array(&mut server, "args", arguments);
            set_string_array(
                &mut server,
                "env_vars",
                &environment.keys().cloned().collect::<Vec<_>>(),
            );
            server["enabled"] = value(*enabled);
            set_timeout(&mut server, *timeout_ms)?;
        }
        ExtensionMcpTransport::Http {
            url,
            headers,
            enabled,
            timeout_ms,
        } => {
            server["url"] = value(url);
            let mut header_variables = toml_edit::InlineTable::new();
            for (header, variable) in headers {
                header_variables.insert(header, Value::from(variable.as_str()));
            }
            if !header_variables.is_empty() {
                server["env_http_headers"] = Item::Value(Value::InlineTable(header_variables));
            }
            server["enabled"] = value(*enabled);
            set_timeout(&mut server, *timeout_ms)?;
        }
    }
    Ok(server)
}

fn set_string_array(table: &mut Table, key: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    let mut array = Array::new();
    for entry in values {
        array.push(entry.as_str());
    }
    table[key] = Item::Value(Value::Array(array));
}

fn set_timeout(table: &mut Table, timeout_ms: Option<u64>) -> Result<(), String> {
    let Some(timeout_ms) = timeout_ms else {
        return Ok(());
    };
    let timeout_secs = timeout_ms.div_ceil(1_000);
    let timeout_secs =
        i64::try_from(timeout_secs).map_err(|_| "MCP 超时时间超出 Codex 支持范围。".to_string())?;
    table["tool_timeout_sec"] = value(timeout_secs);
    Ok(())
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
    let metadata = crate::extensions::skills::skill_installation_metadata(&root);
    visit_skill_directory(&root, &metadata, &mut skills);
    skills
}

fn visit_skill_directory(
    path: &Path,
    metadata: &BTreeMap<String, Option<String>>,
    skills: &mut Vec<AgentItem>,
) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                skills.push(AgentItem {
                    kind: AgentItemKind::Skill,
                    name,
                    version: metadata
                        .get(&entry.file_name().to_string_lossy().to_string())
                        .cloned()
                        .flatten(),
                    source: if metadata
                        .contains_key(&entry.file_name().to_string_lossy().to_string())
                    {
                        AgentItemSource::Team
                    } else {
                        AgentItemSource::Personal
                    },
                    source_path: path.display().to_string(),
                    status: AgentItemStatus::Enabled,
                    error_message: None,
                });
            }
            visit_skill_directory(&path, metadata, skills);
        }
    }
}

pub(crate) fn error_item(kind: AgentItemKind, path: &Path) -> AgentItem {
    AgentItem {
        kind,
        name: "配置读取失败".to_string(),
        version: None,
        source: AgentItemSource::Personal,
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
