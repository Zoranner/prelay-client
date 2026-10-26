use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use serde::{Deserialize, Serialize};

use crate::{
    extensions::{atomic_write, model::HookReviewItem},
    relay::client::ClientError,
};

use super::{merge::storage_error, root::PRELAY_STATE_DIRECTORY};

const HOOK_PACKAGE_STATE_FILE: &str = "hooks.json";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub(super) struct InstalledHookPackages(pub(super) BTreeMap<String, InstalledHookPackage>);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstalledHookPackage {
    pub(super) version: String,
    pub(super) commit_sha: String,
    #[serde(default)]
    pub(super) hashes: BTreeSet<String>,
    #[serde(default)]
    pub(super) review: Vec<HookReviewItem>,
}

pub(super) fn read_installed_hook_packages(
    home: &std::path::Path,
) -> Result<InstalledHookPackages, ClientError> {
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

pub(super) fn write_installed_hook_packages(
    home: &std::path::Path,
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

fn hook_package_state_path(home: &std::path::Path) -> std::path::PathBuf {
    home.join(".codex")
        .join(PRELAY_STATE_DIRECTORY)
        .join(HOOK_PACKAGE_STATE_FILE)
}
