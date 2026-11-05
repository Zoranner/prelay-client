use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::relay::client::ClientError;

use super::ExtensionHost;

pub(crate) struct HostLock {
    file: File,
}

impl Drop for HostLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

pub(crate) fn lock(home: &Path, host: ExtensionHost) -> Result<HostLock, ClientError> {
    let path = lock_path(home, host);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法创建宿主状态目录：{error}"),
            )
        })?;
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法打开宿主状态锁：{error}"),
            )
        })?;
    FileExt::lock_exclusive(&file).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法锁定宿主状态：{error}"),
        )
    })?;
    Ok(HostLock { file })
}

fn lock_path(home: &Path, host: ExtensionHost) -> PathBuf {
    host.root(home).join("state.lock")
}

pub(crate) fn normalize_path(path: &Path) -> String {
    normalize_path_str(&path.display().to_string())
}

pub(crate) fn normalize_path_str(value: &str) -> String {
    let replaced = value.replace('/', "\\");
    if cfg!(windows) {
        replaced.to_lowercase()
    } else {
        replaced
    }
}

pub(crate) fn hash_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}

pub(crate) fn hash_json(value: &Value) -> String {
    hash_bytes(canonical_json(value).as_bytes())
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut entries = map.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            let body = entries
                .into_iter()
                .map(|(key, value)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("JSON key is serializable"),
                        canonical_json(value)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{body}}}")
        }
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => serde_json::to_string(value).expect("JSON value is serializable"),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn valid_plugin_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !matches!(value, "." | "..")
        && !value.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
}

pub(super) fn artifact_identities(artifact: &super::PluginArtifact) -> Vec<String> {
    match artifact {
        super::PluginArtifact::Skill { target, names } => names
            .iter()
            .map(|name| format!("skill:{target}:{name}"))
            .collect(),
        super::PluginArtifact::Mcp {
            target,
            server_name,
            ..
        } => vec![format!("mcp:{target}:{server_name}")],
        super::PluginArtifact::Hook {
            target,
            event,
            matcher,
            handler_hash,
        } => vec![format!(
            "hook:{target}:{event}:{}:{handler_hash}",
            matcher.as_deref().unwrap_or_default()
        )],
    }
}
