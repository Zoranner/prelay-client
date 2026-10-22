use std::path::Path;

use crate::{
    agents::{agent_skill_target_roots, AgentClient},
    relay::client::ClientError,
};

use super::{
    hooks, mcp,
    record::{
        client_key, installed_target, read_installed_extensions, remove_extension_target,
        target_has_members, write_installed_extensions, InstalledTargetRecord,
    },
    skills,
};

const ALL_CLIENTS: [AgentClient; 3] = [
    AgentClient::CodexCli,
    AgentClient::ChatGpt,
    AgentClient::OpenCode,
];

pub(crate) fn uninstall_plugin_for_client(
    home: &Path,
    client: AgentClient,
    package: &str,
) -> Result<(), ClientError> {
    let installed = read_installed_extensions(home)?;
    let Some(record) = installed.0.get(package).cloned() else {
        return Err(ClientError::new(
            "local_extensions_error",
            format!("本机没有扩展包 {package} 的安装记录。"),
        ));
    };
    let Some(target) = record.targets.get(client_key(client)).cloned() else {
        return Err(ClientError::new(
            "local_extensions_error",
            format!("本机没有扩展包 {package} 在当前客户端上的安装记录。"),
        ));
    };
    if record.legacy {
        return uninstall_legacy(home, client, package, installed, target);
    }

    if !target.skills.is_empty() && !shared_by_other_client(home, client, package, "skill")? {
        if let Some(root) = agent_skill_target_roots(&[client], home).into_iter().next() {
            skills::uninstall_skill_package_if_present(&root, package)?;
        }
    }
    if !target.mcps.is_empty() && !shared_by_other_client(home, client, package, "mcp")? {
        mcp::uninstall_mcp_package(home, client, package)?;
    }
    if client == AgentClient::CodexCli && !target.hooks.is_empty() {
        hooks::uninstall_hooks_if_present(home, package)?;
    }
    remove_extension_target(home, client, package)
}

fn uninstall_legacy(
    home: &Path,
    client: AgentClient,
    package: &str,
    mut installed: super::record::InstalledExtensions,
    target: InstalledTargetRecord,
) -> Result<(), ClientError> {
    if !target.skills.is_empty() {
        if let Some(root) = agent_skill_target_roots(&[client], home).into_iter().next() {
            skills::uninstall_skill_package_if_present(&root, package)?;
        }
        if let Some(record) = installed.0.get_mut(package) {
            for target in record.targets.values_mut() {
                target.skills.clear();
            }
        }
    }
    if !target.mcps.is_empty() {
        mcp::uninstall_mcp_package(home, client, package)?;
    }
    if client == AgentClient::CodexCli && !target.hooks.is_empty() {
        hooks::uninstall_hooks_if_present(home, package)?;
    }
    let Some(record) = installed.0.get_mut(package) else {
        return Ok(());
    };
    record.targets.remove(client_key(client));
    record
        .targets
        .retain(|_, target| target_has_members(target));
    if record.targets.is_empty() {
        installed.0.remove(package);
    }
    write_installed_extensions(home, &installed)
}

fn shared_by_other_client(
    home: &Path,
    client: AgentClient,
    package: &str,
    artifact: &str,
) -> Result<bool, ClientError> {
    for other in ALL_CLIENTS.into_iter().filter(|other| *other != client) {
        let Some(target) = installed_target(home, other, package)? else {
            continue;
        };
        let shared = match artifact {
            "skill" => !target.skills.is_empty() && skill_targets_overlap(home, client, other),
            "mcp" => !target.mcps.is_empty() && mcp_host_matches(client, other),
            _ => false,
        };
        if shared {
            return Ok(true);
        }
    }
    Ok(false)
}

fn skill_targets_overlap(home: &Path, left: AgentClient, right: AgentClient) -> bool {
    let left = agent_skill_target_roots(&[left], home);
    let right = agent_skill_target_roots(&[right], home);
    left.iter().any(|path| right.contains(path))
}

fn mcp_host_matches(left: AgentClient, right: AgentClient) -> bool {
    matches!(
        (left, right),
        (AgentClient::CodexCli, AgentClient::ChatGpt)
            | (AgentClient::ChatGpt, AgentClient::CodexCli)
    )
}

#[cfg(test)]
mod tests;
