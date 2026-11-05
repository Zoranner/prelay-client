use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::Deserialize;

use crate::relay::client::ClientError;

use super::LegacySkillRoot;

pub(super) fn read_map<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<BTreeMap<String, T>, ClientError> {
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法读取旧扩展状态 {}：{error}", path.display()),
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(error) => Err(ClientError::new(
            "local_extensions_error",
            format!("无法读取旧扩展状态 {}：{error}", path.display()),
        )),
    }
}

pub(super) fn read_optional<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Option<T>, ClientError> {
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents)
            .map(Some)
            .map_err(|error| {
                ClientError::new(
                    "local_extensions_error",
                    format!("无法读取旧扩展状态 {}：{error}", path.display()),
                )
            }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ClientError::new(
            "local_extensions_error",
            format!("无法读取旧扩展状态 {}：{error}", path.display()),
        )),
    }
}

pub(super) fn read_legacy_skill_roots(
    directory: &Path,
) -> Result<Vec<(String, BTreeSet<String>)>, ClientError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(ClientError::new(
                "local_extensions_error",
                format!("无法读取旧 Skill 状态：{error}"),
            ));
        }
    };
    let mut result = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let legacy: LegacySkillRoot = read_optional(&path)?.ok_or_else(|| {
            ClientError::new("local_extensions_error", "旧 Skill 状态文件不存在。")
        })?;
        if !legacy.package.is_empty() {
            result.push((legacy.package, legacy.roots));
        }
    }
    Ok(result)
}
