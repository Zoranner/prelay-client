use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::relay::client::ClientError;

use super::{atomic_write, storage_error};

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const SKILL_PACKAGE_STATE_FILE: &str = "skill.json";
const LEGACY_MANAGED_SKILLS_DIRECTORY: &str = "skills";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub(super) struct InstalledSkillPackages {
    pub(super) packages: BTreeMap<String, InstalledSkillPackage>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstalledSkillPackage {
    #[serde(default)]
    pub(super) version: Option<String>,
    #[serde(default)]
    pub(super) commit_sha: Option<String>,
    #[serde(default)]
    pub(super) skills: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyManagedSkillPackage {
    #[serde(default)]
    package: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    commit_sha: Option<String>,
    #[serde(default)]
    roots: BTreeSet<String>,
}

pub(super) fn read_installed_skill_packages(
    target_root: &Path,
) -> Result<InstalledSkillPackages, ClientError> {
    let state_path = skill_package_state_path(target_root).ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "skill target root has no parent directory",
        )
    })?;
    match fs::read(&state_path) {
        Ok(contents) => serde_json::from_slice(&contents).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法读取已安装 Skill 状态：{error}"),
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            migrate_legacy_skill_packages(target_root)
        }
        Err(error) => Err(storage_error(error)),
    }
}

pub(super) fn write_installed_skill_packages(
    target_root: &Path,
    packages: &InstalledSkillPackages,
) -> Result<(), ClientError> {
    let contents = serde_json::to_vec(packages).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存已安装 Skill 状态：{error}"),
        )
    })?;
    let path = skill_package_state_path(target_root).ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "skill target root has no parent directory",
        )
    })?;
    atomic_write(&path, &contents)
}

impl InstalledSkillPackages {
    pub(super) fn release_roots_from_other_packages(
        &mut self,
        package: &str,
        roots: &BTreeSet<String>,
    ) {
        self.packages.retain(|name, installed| {
            if name != package {
                installed.skills.retain(|skill| !roots.contains(skill));
            }
            !installed.skills.is_empty()
        });
    }
}

fn skill_package_state_path(target_root: &Path) -> Option<PathBuf> {
    target_root.parent().map(|parent| {
        parent
            .join(PRELAY_STATE_DIRECTORY)
            .join(SKILL_PACKAGE_STATE_FILE)
    })
}

fn migrate_legacy_skill_packages(
    target_root: &Path,
) -> Result<InstalledSkillPackages, ClientError> {
    let Some(parent) = target_root.parent() else {
        return Ok(InstalledSkillPackages::default());
    };
    let legacy_directory = parent
        .join(PRELAY_STATE_DIRECTORY)
        .join(LEGACY_MANAGED_SKILLS_DIRECTORY);
    if !legacy_directory.exists() {
        return Ok(InstalledSkillPackages::default());
    }
    let mut paths = fs::read_dir(&legacy_directory)
        .map_err(storage_error)?
        .map(|entry| entry.map(|entry| entry.path()).map_err(storage_error))
        .collect::<Result<Vec<_>, ClientError>>()?;
    paths.sort();
    let mut installed = InstalledSkillPackages::default();
    for path in paths {
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let contents = fs::read(&path).map_err(storage_error)?;
        let legacy: LegacyManagedSkillPackage =
            serde_json::from_slice(&contents).map_err(|error| {
                ClientError::new(
                    "local_extensions_error",
                    format!("无法迁移已安装 Skill 状态：{error}"),
                )
            })?;
        if legacy.package.is_empty() {
            return Err(ClientError::new(
                "local_extensions_error",
                "无法迁移已安装 Skill 状态：包名为空。",
            ));
        }
        installed.packages.insert(
            legacy.package,
            InstalledSkillPackage {
                version: legacy.version,
                commit_sha: legacy.commit_sha,
                skills: legacy.roots,
            },
        );
    }
    write_installed_skill_packages(target_root, &installed)?;
    fs::remove_dir_all(&legacy_directory).map_err(storage_error)?;
    Ok(installed)
}
