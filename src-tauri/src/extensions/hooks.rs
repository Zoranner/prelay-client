use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionFile;

use crate::{
    agents::{AgentItem, AgentItemKind, AgentItemSource},
    relay::client::ClientError,
};

use super::{
    atomic_write,
    host_state::{ArtifactHealth, HostState, PluginArtifact},
    model::HookReviewItem,
};

mod merge;
mod root;
use merge::{
    add_handler, collect_incoming_hooks, document_hooks, hooks_object_mut, read_hook_document,
    remove_hash,
};
use root::{
    hook_package_root, install_hook_support_files, remove_hook_root, restore_document,
    restore_hook_root,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HookHandler {
    pub event: String,
    pub matcher: Option<String>,
    pub hash: String,
}

pub(crate) struct HookInstall {
    pub handler: HookHandler,
    pub review: HookReviewItem,
}

pub(super) fn install_hooks(
    home: &Path,
    package: &str,
    commit_sha: &str,
    files: &[ExtensionFile],
    previous_hashes: &BTreeSet<String>,
) -> Result<Vec<HookInstall>, ClientError> {
    if files.is_empty() {
        return Err(ClientError::new("invalid_response", "Hook 安装包为空。"));
    }
    let root = hook_package_root(home, package, commit_sha)?;
    let incoming = collect_incoming_hooks(files, Some(&root))?;
    let hooks_path = hooks_path(home);
    let original_contents = fs::read(&hooks_path).ok();
    let mut document = read_hook_document(&hooks_path)?;
    let backup = install_hook_support_files(&root, files)?;

    let merged = (|| {
        let hooks = hooks_object_mut(&mut document)?;
        for hash in previous_hashes {
            remove_hash(hooks, hash);
        }
        for entry in &incoming {
            remove_hash(hooks, &entry.hash);
            add_handler(
                hooks,
                &entry.event,
                entry.matcher.as_deref(),
                entry.handler.clone(),
            );
        }
        let contents = serde_json::to_vec_pretty(&document).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法序列化 Codex Hook 配置：{error}"),
            )
        })?;
        atomic_write(&hooks_path, &contents)
    })();
    if let Err(error) = merged {
        restore_document(&hooks_path, original_contents.as_deref());
        restore_hook_root(&root, backup);
        return Err(error);
    }
    if let Some(backup) = backup {
        let _ = fs::remove_dir_all(backup);
    }
    Ok(incoming
        .into_iter()
        .map(|entry| HookInstall {
            handler: HookHandler {
                event: entry.event,
                matcher: entry.matcher,
                hash: entry.hash,
            },
            review: entry.review,
        })
        .collect())
}

pub(crate) fn validate_hook_files(files: &[ExtensionFile]) -> Result<(), ClientError> {
    collect_incoming_hooks(files, None).map(|_| ())
}

pub(crate) fn plan_hooks(
    home: &Path,
    package: &str,
    commit_sha: &str,
    files: &[ExtensionFile],
) -> Result<Vec<HookHandler>, ClientError> {
    let root = hook_package_root(home, package, commit_sha)?;
    Ok(collect_incoming_hooks(files, Some(&root))?
        .into_iter()
        .map(|entry| HookHandler {
            event: entry.event,
            matcher: entry.matcher,
            hash: entry.hash,
        })
        .collect())
}

pub(crate) fn preflight_hook_install(
    home: &Path,
    package: &str,
    incoming: &[HookHandler],
    overwrite: bool,
    state: &HostState,
) -> Result<(), ClientError> {
    if incoming.is_empty() {
        return Ok(());
    }
    let previous = state
        .plugin(package)
        .into_iter()
        .flat_map(|plugin| plugin.hooks())
        .filter_map(|artifact| match artifact {
            PluginArtifact::Hook { handler_hash, .. } => Some(handler_hash.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let owned_by_other = state
        .active_plugins()
        .filter(|plugin| plugin.name != package)
        .flat_map(|plugin| plugin.hooks())
        .filter_map(|artifact| match artifact {
            PluginArtifact::Hook { handler_hash, .. } => Some(handler_hash.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let existing = self::handlers(home)?
        .into_iter()
        .map(|handler| handler.hash)
        .collect::<BTreeSet<_>>();
    for handler in incoming {
        if owned_by_other.contains(&handler.hash) {
            return Err(ClientError::new(
                "extension_target_exists",
                "Hook 已由另一个扩展包管理。",
            ));
        }
        if !overwrite && existing.contains(&handler.hash) && !previous.contains(&handler.hash) {
            return Err(ClientError::new(
                "extension_target_exists",
                "Hook 已存在，确认后可覆盖安装。",
            ));
        }
    }
    Ok(())
}

pub(crate) fn uninstall_hooks(home: &Path, hashes: &BTreeSet<String>) -> Result<(), ClientError> {
    if hashes.is_empty() {
        return Ok(());
    }
    let hooks_path = hooks_path(home);
    if !hooks_path.exists() {
        return Ok(());
    }
    let mut document = read_hook_document(&hooks_path)?;
    let hooks = hooks_object_mut(&mut document)?;
    for hash in hashes {
        remove_hash(hooks, hash);
    }
    let contents = serde_json::to_vec_pretty(&document).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法序列化 Codex Hook 配置：{error}"),
        )
    })?;
    atomic_write(&hooks_path, &contents)
}

pub(crate) fn handlers(home: &Path) -> Result<Vec<HookHandler>, ClientError> {
    let document = read_hook_document(&hooks_path(home))?;
    Ok(document_hooks(&document)
        .into_iter()
        .map(|hook| HookHandler {
            event: hook.event,
            matcher: hook.matcher,
            hash: hook.hash,
        })
        .collect())
}

pub(crate) fn health(
    home: &Path,
    event: &str,
    matcher: Option<&str>,
    handler_hash: &str,
) -> Result<ArtifactHealth, ClientError> {
    let present = handlers(home)?.into_iter().any(|handler| {
        handler.event == event
            && handler.matcher.as_deref() == matcher
            && handler.hash == handler_hash
    });
    Ok(if present {
        ArtifactHealth::Managed
    } else {
        ArtifactHealth::Missing
    })
}

pub(crate) fn unmanaged_hook_items(
    home: &Path,
    state: &HostState,
) -> Result<Vec<AgentItem>, ClientError> {
    let hooks_path = hooks_path(home);
    let document = read_hook_document(&hooks_path)?;
    let owned = state
        .active_plugins()
        .flat_map(|plugin| plugin.hooks())
        .filter_map(|artifact| match artifact {
            PluginArtifact::Hook { handler_hash, .. } => Some(handler_hash.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut items = Vec::new();
    for hook in document_hooks(&document) {
        if owned.contains(&hook.hash) {
            continue;
        }
        items.push(AgentItem {
            kind: AgentItemKind::Hook,
            name: hook.event,
            version: None,
            package: None,
            hook_hash: Some(hook.hash),
            matcher: hook.matcher,
            command: Some(hook.command),
            transport: None,
            target: None,
            description: None,
            file_count: None,
            members: Vec::new(),
            artifacts: Vec::new(),
            source: AgentItemSource::Personal,
            source_path: hooks_path.display().to_string(),
            error_message: None,
        });
    }
    Ok(items)
}

pub(crate) fn remove_hook_handler(home: &Path, hash: &str) -> Result<(), ClientError> {
    uninstall_hooks(home, &BTreeSet::from([hash.to_string()]))
}

pub(crate) fn remove_package_root(
    home: &Path,
    package: &str,
    commit_sha: &str,
) -> Result<(), ClientError> {
    remove_hook_root(home, package, commit_sha)
}

fn hooks_path(home: &Path) -> PathBuf {
    home.join(".codex").join("hooks.json")
}
