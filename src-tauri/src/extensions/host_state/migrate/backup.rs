use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::relay::client::ClientError;

pub(super) fn backup_legacy(home: &Path) -> Result<PathBuf, ClientError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let root = home
        .join(".prelay")
        .join("backup")
        .join(timestamp.to_string());
    fs::create_dir_all(&root).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法创建迁移备份目录：{error}"),
        )
    })?;
    for (source, name) in [
        (
            home.join(".prelay").join("extensions.json"),
            "extensions.json",
        ),
        (
            home.join(".codex").join(".prelay").join("hooks.json"),
            "hooks.json",
        ),
        (
            home.join(".codex").join(".prelay").join("mcp.json"),
            "mcp.json",
        ),
        (
            home.join(".codex").join(".prelay").join("rule.json"),
            "rule.json",
        ),
        (
            home.join(".agents").join(".prelay").join("skill.json"),
            "skill.json",
        ),
    ] {
        if source.exists() {
            fs::copy(&source, root.join(name)).map_err(|error| {
                ClientError::new(
                    "local_extensions_error",
                    format!("无法备份旧扩展状态：{error}"),
                )
            })?;
        }
    }
    let legacy_skills = home.join(".agents").join(".prelay").join("skills");
    if legacy_skills.exists() {
        copy_directory(&legacy_skills, &root.join("skills"))?;
    }
    Ok(root)
}

fn copy_directory(source: &Path, target: &Path) -> Result<(), ClientError> {
    fs::create_dir_all(target).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法创建迁移备份目录：{error}"),
        )
    })?;
    let entries = fs::read_dir(source).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法读取旧 Skill 状态：{error}"),
        )
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        let destination = target.join(entry.file_name());
        if path.is_dir() {
            copy_directory(&path, &destination)?;
        } else {
            fs::copy(&path, &destination).map_err(|error| {
                ClientError::new(
                    "local_extensions_error",
                    format!("无法备份旧 Skill 状态：{error}"),
                )
            })?;
        }
    }
    Ok(())
}
