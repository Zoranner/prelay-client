use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionFile;
use serde::{Deserialize, Serialize};

use crate::{
    agents::{AgentClient, AgentItem, AgentItemKind, AgentItemSource, AgentItemStatus},
    relay::client::ClientError,
};

use super::{atomic_write, model::HookReviewItem, ExtensionInstallAction};

mod merge;
use merge::{
    add_handler, collect_incoming_hooks, document_hashes, hooks_object_mut, read_hook_document,
    remove_hash, storage_error,
};

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const HOOK_PACKAGE_STATE_FILE: &str = "hooks.json";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
struct InstalledHookPackages(BTreeMap<String, InstalledHookPackage>);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledHookPackage {
    version: String,
    commit_sha: String,
    #[serde(default)]
    hashes: BTreeSet<String>,
    #[serde(default)]
    review: Vec<HookReviewItem>,
}

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

    let hooks_path = hooks_path(home);
    let mut document = read_hook_document(&hooks_path)?;
    let incoming = collect_incoming_hooks(files)?;
    let mut installed = read_installed_hook_packages(home)?;
    let previous = installed.0.get(package).cloned();
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

    let contents = serde_json::to_vec_pretty(&document).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法序列化 Codex Hook 配置：{error}"),
        )
    })?;
    atomic_write(&hooks_path, &contents)?;
    write_installed_hook_packages(home, &installed)?;
    Ok(review)
}

pub(crate) fn validate_hook_files(files: &[ExtensionFile]) -> Result<(), ClientError> {
    collect_incoming_hooks(files).map(|_| ())
}

pub(crate) fn preflight_hooks(
    home: &Path,
    clients: &[AgentClient],
    files: &[ExtensionFile],
) -> Result<(), ClientError> {
    if !clients.contains(&AgentClient::CodexCli) {
        return Ok(());
    }
    read_hook_document(&hooks_path(home))?;
    collect_incoming_hooks(files).map(|_| ())
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
    let missing = current.hashes.iter().any(|hash| !present.contains(hash));
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

fn hook_package_state_path(home: &Path) -> PathBuf {
    home.join(".codex")
        .join(PRELAY_STATE_DIRECTORY)
        .join(HOOK_PACKAGE_STATE_FILE)
}

fn read_installed_hook_packages(home: &Path) -> Result<InstalledHookPackages, ClientError> {
    let path = hook_package_state_path(home);
    match fs::read(&path) {
        Ok(contents) => serde_json::from_slice(&contents).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法读取已安装 Hook 状态：{error}"),
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(InstalledHookPackages::default())
        }
        Err(error) => Err(storage_error(error)),
    }
}

fn write_installed_hook_packages(
    home: &Path,
    packages: &InstalledHookPackages,
) -> Result<(), ClientError> {
    let contents = serde_json::to_vec(packages).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存已安装 Hook 状态：{error}"),
        )
    })?;
    atomic_write(&hook_package_state_path(home), &contents)
}

#[cfg(test)]
mod tests;
