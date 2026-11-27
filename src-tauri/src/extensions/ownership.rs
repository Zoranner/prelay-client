use std::path::Path;

use crate::relay::client::ClientError;

use super::host_state::{normalize_path, HostState, PluginArtifact};

/// 目标制品的归属判定结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Owner<'a> {
    /// 本包记录已声明。
    Owned,
    /// 另一个已安装插件声明。
    Other(&'a str),
    /// 没有任何记录声明。
    None,
}

pub(crate) fn skill_owner<'a>(
    state: &'a HostState,
    package: &str,
    target_root: &Path,
    name: &str,
) -> Owner<'a> {
    let target = normalize_path(target_root);
    owner_of(state, package, |artifact| match artifact {
        PluginArtifact::Skill {
            target: artifact_target,
            names,
        } => artifact_target == &target && names.iter().any(|candidate| candidate == name),
        _ => false,
    })
}

pub(crate) fn mcp_owner<'a>(state: &'a HostState, package: &str, server_name: &str) -> Owner<'a> {
    owner_of(state, package, |artifact| {
        matches!(
            artifact,
            PluginArtifact::Mcp { server_name: artifact_name, .. } if artifact_name == server_name
        )
    })
}

pub(crate) fn hook_owner<'a>(state: &'a HostState, package: &str, handler_hash: &str) -> Owner<'a> {
    owner_of(state, package, |artifact| {
        matches!(
            artifact,
            PluginArtifact::Hook { handler_hash: artifact_hash, .. } if artifact_hash == handler_hash
        )
    })
}

pub(crate) fn rule_owner<'a>(state: &'a HostState, package: &str, target: &Path) -> Owner<'a> {
    match state.rule_entry(target) {
        Some(entry) if entry.package == package => Owner::Owned,
        Some(entry) => Owner::Other(entry.package.as_str()),
        None => Owner::None,
    }
}

fn owner_of<'a>(
    state: &'a HostState,
    package: &str,
    matches: impl Fn(&PluginArtifact) -> bool,
) -> Owner<'a> {
    let owned = state
        .plugin(package)
        .is_some_and(|plugin| plugin.artifacts.iter().any(&matches));
    if owned {
        return Owner::Owned;
    }
    for plugin in state.active_plugins() {
        if plugin.name == package {
            continue;
        }
        if plugin.artifacts.iter().any(&matches) {
            return Owner::Other(plugin.name.as_str());
        }
    }
    Owner::None
}

pub(crate) fn owned_elsewhere(package: &str) -> ClientError {
    ClientError::new(
        "local_extensions_conflict",
        format!("目标由扩展包 {package} 管理，请先卸载该扩展包。"),
    )
}

pub(crate) fn unmanaged_exists() -> ClientError {
    ClientError::new(
        "extension_target_exists",
        "目标位置已有未受管内容，确认后可覆盖安装。",
    )
}
