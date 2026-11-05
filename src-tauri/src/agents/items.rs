use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use toml_edit::DocumentMut;

use crate::extensions::{
    hooks::{remove_hook_handler, unmanaged_hook_items},
    host_state::{ExtensionHost, HostState, PluginState, StateError},
    mcp::managed_mcp_servers,
    uninstall::uninstall_plugin_for_client,
};

use super::{
    discovery::agent_client_is_installed,
    integrations::integration,
    model::{
        AgentClient, AgentClientItems, AgentItem, AgentItemKind, AgentItemSource,
        AgentItemsSnapshot, REGISTERED_AGENT_CLIENTS,
    },
};

mod details;
mod skill_scan;
use details::{plugin_artifact_groups, read_toml, toml_items};

pub fn scan_user_items(home: &Path) -> AgentItemsSnapshot {
    scan_user_items_with_installation(home, agent_client_is_installed)
}

pub fn scan_agent_items(home: &Path, client: AgentClient) -> AgentClientItems {
    AgentClientItems {
        client,
        version: None,
        items: scan_client_items(home, client),
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

pub(crate) fn user_skill_roots(home: &Path) -> [PathBuf; 2] {
    [
        home.join(".codex").join("skills"),
        home.join(".agents").join("skills"),
    ]
}

pub(crate) fn scan_user_items_with_installation(
    home: &Path,
    is_installed: impl Fn(AgentClient) -> bool,
) -> AgentItemsSnapshot {
    let mut snapshot = AgentItemsSnapshot::default();
    for client in REGISTERED_AGENT_CLIENTS {
        if is_installed(client) {
            snapshot.clients.push(AgentClientItems {
                client,
                version: None,
                items: scan_client_items(home, client),
            });
        }
    }
    snapshot
}

fn scan_client_items(home: &Path, client: AgentClient) -> Vec<AgentItem> {
    let state = match HostState::load(home, ExtensionHost::from_client(client)) {
        Ok(state) => state,
        Err(error) => {
            return vec![error_item(
                AgentItemKind::Plugin,
                &home.join(".prelay").join("hosts"),
                &error.message,
            )];
        }
    };
    let managed = state.plugin_names();
    let mut items = integration(client)
        .scan(home)
        .into_iter()
        .filter(|item| {
            item.package
                .as_deref()
                .is_none_or(|package| !managed.contains(package))
        })
        .collect::<Vec<_>>();
    items.extend(plugin_items(&state, home));
    items.extend(state.errors.iter().map(state_error_item));
    items
}

pub fn uninstall_user_item(
    home: &Path,
    client: AgentClient,
    kind: AgentItemKind,
    name: &str,
    source_path: &str,
    hook_hash: Option<&str>,
) -> Result<(), String> {
    uninstall_user_item_with_installation(
        home,
        client,
        kind,
        name,
        source_path,
        hook_hash,
        agent_client_is_installed,
    )
}

pub(crate) fn uninstall_user_item_with_installation(
    home: &Path,
    client: AgentClient,
    kind: AgentItemKind,
    name: &str,
    source_path: &str,
    hook_hash: Option<&str>,
    is_installed: impl Fn(AgentClient) -> bool,
) -> Result<(), String> {
    let item = scan_user_items_with_installation(home, is_installed)
        .clients
        .into_iter()
        .find(|items| items.client == client)
        .and_then(|items| {
            items.items.into_iter().find(|item| {
                item.kind == kind
                    && item.name == name
                    && item.source_path == source_path
                    && (kind != AgentItemKind::Hook
                        || hook_hash.is_none()
                        || item.hook_hash.as_deref() == hook_hash)
            })
        })
        .ok_or_else(|| "未找到要卸载的本地条目。".to_string())?;

    if item.source == AgentItemSource::System {
        return Err("内置技能不可卸载。".to_string());
    }
    if let Some(package) = item.package.as_deref() {
        return uninstall_plugin_for_client(home, client, package).map_err(|error| error.message);
    }
    if kind == AgentItemKind::Hook {
        if let Some(hash) = item.hook_hash.as_deref() {
            return remove_hook_handler(home, hash).map_err(|error| error.message);
        }
    }
    integration(client).uninstall(home, kind, name, &item.source_path)
}

pub(crate) fn scan_codex(home: &Path) -> Vec<AgentItem> {
    let codex_root = home.join(".codex");
    let config_path = codex_root.join("config.toml");
    let state = match HostState::load(home, ExtensionHost::Codex) {
        Ok(state) => state,
        Err(error) => {
            return vec![error_item(
                AgentItemKind::Plugin,
                &codex_root,
                &error.message,
            )]
        }
    };
    let managed_mcp = managed_mcp_servers(&state);
    let mut items = if codex_root.exists() {
        match read_toml(&config_path) {
            Ok(Some(value)) => toml_items(
                &value,
                "mcp_servers",
                AgentItemKind::Mcp,
                &config_path,
                &managed_mcp,
            ),
            Ok(None) => Vec::new(),
            Err(()) => vec![error_item(
                AgentItemKind::Mcp,
                &config_path,
                "无法读取扩展配置。",
            )],
        }
    } else {
        Vec::new()
    };
    for root in user_skill_roots(home) {
        items.extend(skill_scan::scan_skills(root, &state));
    }
    mark_codex_system_skills(&mut items, home);
    match unmanaged_hook_items(home, &state) {
        Ok(hooks) => items.extend(hooks),
        Err(error) => items.push(error_item(
            AgentItemKind::Hook,
            &codex_root.join("hooks.json"),
            &error.message,
        )),
    }
    deduplicate(items)
}

fn plugin_items(state: &HostState, home: &Path) -> Vec<AgentItem> {
    let source_path = |name: &str| {
        ExtensionHost::Codex
            .plugin_path(home, name)
            .display()
            .to_string()
    };
    state
        .plugins
        .iter()
        .filter_map(|plugin| {
            let members = plugin.members();
            if members.is_empty() {
                return None;
            }
            Some(AgentItem {
                kind: AgentItemKind::Plugin,
                name: plugin.name.clone(),
                version: Some(plugin.version.clone()),
                package: Some(plugin.name.clone()),
                hook_hash: None,
                matcher: None,
                command: None,
                transport: None,
                target: None,
                description: None,
                file_count: None,
                members,
                artifacts: plugin_artifact_groups(plugin),
                source: AgentItemSource::Team,
                source_path: source_path(&plugin.name),
                error_message: if state.is_conflicted(&plugin.name) {
                    Some("扩展制品归属冲突，未参与管理。".to_string())
                } else {
                    (plugin.state != PluginState::Installed).then(|| "扩展安装未完成。".to_string())
                },
            })
        })
        .collect()
}

fn state_error_item(error: &StateError) -> AgentItem {
    AgentItem {
        kind: AgentItemKind::Plugin,
        name: "扩展状态读取失败".to_string(),
        version: None,
        package: None,
        hook_hash: None,
        matcher: None,
        command: None,
        transport: None,
        target: None,
        description: None,
        file_count: None,
        members: Vec::new(),
        artifacts: Vec::new(),
        source: AgentItemSource::Personal,
        source_path: error.path.clone(),
        error_message: Some(error.message.clone()),
    }
}

fn mark_codex_system_skills(items: &mut [AgentItem], home: &Path) {
    let system_root = home.join(".codex").join("skills").join(".system");
    for item in items {
        if item.kind == AgentItemKind::Skill
            && Path::new(&item.source_path).starts_with(&system_root)
        {
            item.source = AgentItemSource::System;
        }
    }
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

pub(crate) fn remove_skill_directory(source_path: &str) -> Result<(), String> {
    remove_directory(Path::new(source_path))
}

fn remove_directory(path: &Path) -> Result<(), String> {
    fs::remove_dir_all(path).map_err(|error| format!("无法删除本地文件：{error}"))
}

pub(crate) fn write_text(path: &Path, contents: &[u8]) -> Result<(), String> {
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

pub(crate) fn error_item(kind: AgentItemKind, path: &Path, message: &str) -> AgentItem {
    AgentItem {
        kind,
        name: "配置读取失败".to_string(),
        version: None,
        package: None,
        hook_hash: None,
        matcher: None,
        command: None,
        transport: None,
        target: None,
        description: None,
        file_count: None,
        members: Vec::new(),
        artifacts: Vec::new(),
        source: AgentItemSource::Personal,
        source_path: path.display().to_string(),
        error_message: Some(message.to_string()),
    }
}

pub(crate) fn deduplicate(mut items: Vec<AgentItem>) -> Vec<AgentItem> {
    items.sort_by(|left, right| {
        (
            left.kind as u8,
            left.name.as_str(),
            left.source_path.as_str(),
            left.hook_hash.as_deref(),
        )
            .cmp(&(
                right.kind as u8,
                right.name.as_str(),
                right.source_path.as_str(),
                right.hook_hash.as_deref(),
            ))
    });
    items.dedup_by(|left, right| {
        left.kind == right.kind
            && left.name == right.name
            && left.source_path == right.source_path
            && left.hook_hash == right.hook_hash
    });
    items
}
