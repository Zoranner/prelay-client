use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionFile;

use crate::{
    agents::{AgentClient, AgentItem, AgentItemKind, AgentItemSource, AgentItemStatus},
    relay::client::ClientError,
};

use super::{atomic_write, model::HookReviewItem, ExtensionInstallAction};

mod merge;
mod root;
mod state;
use merge::{
    add_handler, collect_incoming_hooks, document_hashes, hooks_object_mut, read_hook_document,
    remove_hash,
};
use root::{
    hook_package_root, install_hook_support_files, remove_hook_root, restore_document,
    restore_hook_root,
};
use state::{read_installed_hook_packages, write_installed_hook_packages, InstalledHookPackage};

pub(crate) struct HookInstallationStatus {
    pub action: ExtensionInstallAction,
    pub clients: Vec<AgentClient>,
}

pub(super) fn install_hooks(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    version: &str,
    commit_sha: &str,
    files: &[ExtensionFile],
    _overwrite: bool,
) -> Result<Vec<HookReviewItem>, ClientError> {
    if !clients.contains(&AgentClient::CodexCli) {
        return Ok(Vec::new());
    }
    if files.is_empty() {
        return Err(ClientError::new("invalid_response", "Hook 安装包为空。"));
    }

    let root = hook_package_root(home, package, commit_sha)?;
    let incoming = collect_incoming_hooks(files, Some(&root))?;
    let hooks_path = hooks_path(home);
    let original_contents = fs::read(&hooks_path).ok();
    let mut document = read_hook_document(&hooks_path)?;
    let mut installed = read_installed_hook_packages(home)?;
    let previous = installed.0.get(package).cloned();
    let backup = install_hook_support_files(&root, files)?;

    let merged = (|| {
        let hooks = hooks_object_mut(&mut document)?;
        if let Some(previous) = &previous {
            for hash in &previous.hashes {
                remove_hash(hooks, hash);
            }
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

    let hashes = incoming
        .iter()
        .map(|entry| entry.hash.clone())
        .collect::<BTreeSet<_>>();
    let review = incoming
        .iter()
        .map(|entry| entry.review.clone())
        .collect::<Vec<_>>();
    installed.0.insert(
        package.to_string(),
        InstalledHookPackage {
            version: version.to_string(),
            commit_sha: commit_sha.to_string(),
            hashes,
            review: review.clone(),
        },
    );

    if let Err(error) = write_installed_hook_packages(home, &installed) {
        restore_document(&hooks_path, original_contents.as_deref());
        restore_hook_root(&root, backup);
        return Err(error);
    }
    if let Some(backup) = backup {
        let _ = fs::remove_dir_all(backup);
    }
    if let Some(previous) = previous.filter(|previous| previous.commit_sha != commit_sha) {
        remove_hook_root(home, package, &previous.commit_sha)?;
    }
    Ok(review)
}

pub(crate) fn validate_hook_files(files: &[ExtensionFile]) -> Result<(), ClientError> {
    collect_incoming_hooks(files, None).map(|_| ())
}

pub(super) fn preview_hooks(
    home: &Path,
    package: &str,
    commit_sha: &str,
    files: &[ExtensionFile],
) -> Result<Vec<HookReviewItem>, ClientError> {
    let root = hook_package_root(home, package, commit_sha)?;
    Ok(collect_incoming_hooks(files, Some(&root))?
        .into_iter()
        .map(|entry| entry.review)
        .collect())
}

pub(crate) fn preflight_hooks(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    commit_sha: &str,
    files: &[ExtensionFile],
) -> Result<(), ClientError> {
    if !clients.contains(&AgentClient::CodexCli) {
        return Ok(());
    }
    let root = hook_package_root(home, package, commit_sha)?;
    read_hook_document(&hooks_path(home))?;
    collect_incoming_hooks(files, Some(&root)).map(|_| ())
}

pub(crate) fn hook_installation_status(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    version: &str,
    commit_sha: &str,
) -> Result<HookInstallationStatus, ClientError> {
    if !clients.contains(&AgentClient::CodexCli) {
        return Ok(HookInstallationStatus {
            action: ExtensionInstallAction::Install,
            clients: Vec::new(),
        });
    }
    let installed = read_installed_hook_packages(home)?;
    let Some(current) = installed.0.get(package) else {
        return Ok(HookInstallationStatus {
            action: ExtensionInstallAction::Install,
            clients: Vec::new(),
        });
    };
    let document = read_hook_document(&hooks_path(home))?;
    let present = document_hashes(&document);
    let root_missing = !hook_package_root(home, package, &current.commit_sha)?.is_dir();
    let missing = root_missing || current.hashes.iter().any(|hash| !present.contains(hash));
    if missing {
        return Ok(HookInstallationStatus {
            action: ExtensionInstallAction::Partial,
            clients: Vec::new(),
        });
    }
    let outdated = current.version != version || current.commit_sha != commit_sha;
    Ok(HookInstallationStatus {
        action: if outdated {
            ExtensionInstallAction::Update
        } else {
            ExtensionInstallAction::Installed
        },
        clients: vec![AgentClient::CodexCli],
    })
}

pub(crate) fn uninstall_hooks(home: &Path, package: &str) -> Result<(), ClientError> {
    let mut installed = read_installed_hook_packages(home)?;
    let Some(current) = installed.0.remove(package) else {
        return Err(ClientError::new(
            "local_extensions_error",
            format!("本机没有扩展包 {package} 的 Hook 安装记录。"),
        ));
    };
    let hooks_path = hooks_path(home);
    if hooks_path.exists() {
        let mut document = read_hook_document(&hooks_path)?;
        let present = document_hashes(&document);
        if current.hashes.iter().any(|hash| !present.contains(hash)) {
            return Err(ClientError::new(
                "local_extensions_conflict",
                format!("扩展包 {package} 的 Hook 条目已被手动修改，未删除本机安装记录。"),
            ));
        }
        let hooks = hooks_object_mut(&mut document)?;
        for hash in &current.hashes {
            remove_hash(hooks, hash);
        }
        let contents = serde_json::to_vec_pretty(&document).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法序列化 Codex Hook 配置：{error}"),
            )
        })?;
        atomic_write(&hooks_path, &contents)?;
    }
    remove_hook_root(home, package, &current.commit_sha)?;
    write_installed_hook_packages(home, &installed)
}

pub(crate) fn uninstall_hooks_if_present(home: &Path, package: &str) -> Result<(), ClientError> {
    let installed = read_installed_hook_packages(home)?;
    if installed.0.contains_key(package) {
        uninstall_hooks(home, package)?;
    }
    Ok(())
}

pub(crate) fn managed_hook_items(home: &Path) -> Vec<AgentItem> {
    let Ok(installed) = read_installed_hook_packages(home) else {
        return Vec::new();
    };
    let source_path = hooks_path(home).display().to_string();
    installed
        .0
        .into_iter()
        .map(|(package, entry)| AgentItem {
            kind: AgentItemKind::Hook,
            name: package.clone(),
            version: Some(entry.version),
            package: Some(package),
            members: entry
                .review
                .iter()
                .map(|review| match &review.matcher {
                    Some(matcher) => format!("{}:{}", review.event, matcher),
                    None => review.event.clone(),
                })
                .collect(),
            source: AgentItemSource::Team,
            source_path: source_path.clone(),
            status: AgentItemStatus::Enabled,
            error_message: None,
        })
        .collect()
}

pub(crate) fn retain_listed_hook_packages(
    home: &Path,
    listed_packages: &BTreeSet<String>,
) -> Result<(), ClientError> {
    let mut installed = read_installed_hook_packages(home)?;
    let initial_len = installed.0.len();
    installed
        .0
        .retain(|package, _| listed_packages.contains(package));
    if installed.0.len() != initial_len {
        write_installed_hook_packages(home, &installed)?;
    }
    Ok(())
}

fn hooks_path(home: &Path) -> PathBuf {
    home.join(".codex").join("hooks.json")
}

#[cfg(test)]
mod root_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
