use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use prelay_protocol::ExtensionFile;

use crate::{
    extensions::{atomic_write, decode_extension_file},
    relay::client::ClientError,
};

use super::merge::storage_error;

pub(super) const HOOKS_PREFIX: &str = "hooks/";
pub(super) const PRELAY_STATE_DIRECTORY: &str = ".prelay";
pub(super) const PACKAGES_DIRECTORY: &str = "packages";

pub(super) fn hook_package_root(
    home: &Path,
    package: &str,
    commit_sha: &str,
) -> Result<PathBuf, ClientError> {
    if !safe_segment(package) || !safe_segment(commit_sha) {
        return Err(ClientError::new(
            "invalid_response",
            "扩展包名或版本标识包含无效路径。",
        ));
    }
    Ok(home
        .join(PRELAY_STATE_DIRECTORY)
        .join(PACKAGES_DIRECTORY)
        .join(package)
        .join(commit_sha)
        .join("hooks"))
}

fn safe_segment(value: &str) -> bool {
    !value.is_empty() && !matches!(value, "." | "..") && !value.contains(['/', '\\', ':'])
}

fn hook_relative_path(path: &str) -> Result<&str, ClientError> {
    let relative = path
        .strip_prefix(HOOKS_PREFIX)
        .ok_or_else(|| ClientError::new("invalid_response", "Hook 安装包包含无效路径。"))?;
    if relative.is_empty()
        || path.contains(['\\', '\0'])
        || relative
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(ClientError::new(
            "invalid_response",
            "Hook 安装包包含无效路径。",
        ));
    }
    Ok(relative)
}

pub(super) fn install_hook_support_files(
    root: &Path,
    files: &[ExtensionFile],
) -> Result<Option<PathBuf>, ClientError> {
    let parent = root
        .parent()
        .ok_or_else(|| ClientError::new("local_extensions_error", "Hook 受管根缺少父目录。"))?;
    let staging = parent.join(format!(".hooks-staging-{}", unique_hook_suffix()));
    let staged = (|| {
        for file in files {
            let relative = hook_relative_path(&file.path)?;
            atomic_write(&staging.join(relative), &decode_extension_file(file)?)?;
        }
        Ok(())
    })();
    if let Err(error) = staged {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    match replace_hook_root(root, &staging) {
        Ok(backup) => Ok(backup),
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            Err(error)
        }
    }
}

fn replace_hook_root(root: &Path, staging: &Path) -> Result<Option<PathBuf>, ClientError> {
    let parent = root
        .parent()
        .ok_or_else(|| ClientError::new("local_extensions_error", "Hook 受管根缺少父目录。"))?;
    let backup = if root.exists() {
        let backup = parent.join(format!(".hooks-backup-{}", unique_hook_suffix()));
        fs::rename(root, &backup).map_err(storage_error)?;
        Some(backup)
    } else {
        None
    };
    match fs::rename(staging, root) {
        Ok(()) => Ok(backup),
        Err(error) => {
            if let Some(backup) = &backup {
                let _ = fs::rename(backup, root);
            }
            Err(storage_error(error))
        }
    }
}

pub(super) fn restore_hook_root(root: &Path, backup: Option<PathBuf>) {
    let _ = fs::remove_dir_all(root);
    if let Some(backup) = backup {
        let _ = fs::rename(backup, root);
    }
}

pub(super) fn restore_document(path: &Path, contents: Option<&[u8]>) {
    match contents {
        Some(contents) => {
            let _ = atomic_write(path, contents);
        }
        None => {
            let _ = fs::remove_file(path);
        }
    }
}

fn unique_hook_suffix() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

pub(super) fn remove_hook_root(
    home: &Path,
    package: &str,
    commit_sha: &str,
) -> Result<(), ClientError> {
    let root = hook_package_root(home, package, commit_sha)?;
    let commit_root = root
        .parent()
        .ok_or_else(|| ClientError::new("local_extensions_error", "Hook 受管根缺少父目录。"))?;
    match fs::remove_dir_all(commit_root) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(storage_error(error)),
    }
}
