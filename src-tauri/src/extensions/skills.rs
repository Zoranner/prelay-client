use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionFile;
use serde::{Deserialize, Serialize};

use crate::{agents::AgentClient, relay::client::ClientError};

use super::{
    atomic_write, decode_extension_file, storage_error, ExtensionInstallAction, ExtensionPackage,
    SKILLS_PREFIX,
};

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const SKILL_PACKAGE_STATE_FILE: &str = "skill.json";
const LEGACY_MANAGED_SKILLS_DIRECTORY: &str = "skills";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct InstalledSkillPackages(BTreeMap<String, InstalledSkillPackage>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledSkillPackage {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub commit_sha: Option<String>,
    #[serde(default)]
    pub skills: BTreeSet<String>,
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

struct SkillInstallContents<'a> {
    files: Vec<(&'a ExtensionFile, Vec<u8>)>,
    roots: BTreeSet<String>,
    removed_roots: BTreeSet<String>,
}

pub(super) fn install_skill_files(
    target_root: &Path,
    package: &str,
    version: &str,
    commit_sha: &str,
    files: &[ExtensionFile],
    overwrite: bool,
) -> Result<(), ClientError> {
    if package.is_empty() {
        return Err(ClientError::new(
            "invalid_response",
            "skill package name is empty",
        ));
    }
    let contents = skill_install_contents(files)?;
    if !contents.roots.is_disjoint(&contents.removed_roots) {
        return Err(ClientError::new(
            "invalid_response",
            "skill install bundle cannot replace and delete the same skill",
        ));
    }
    let mut installed = read_installed_skill_packages(target_root)?;

    if !overwrite {
        ensure_skill_roots_available(target_root, package, &contents.roots, &installed)?;
    }
    if overwrite {
        installed.release_roots_from_other_packages(package, &contents.roots);
    }

    for root in &contents.roots {
        let path = target_root.join(root);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(storage_error)?;
        }
    }

    for (source, content) in contents.files {
        let relative = source
            .path
            .strip_prefix(SKILLS_PREFIX)
            .expect("validated skill path");
        atomic_write(&target_root.join(relative), &content)?;
    }
    for root in contents.removed_roots {
        let path = target_root.join(root);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(storage_error)?;
        }
    }
    installed.0.insert(
        package.to_string(),
        InstalledSkillPackage {
            version: Some(version.to_string()),
            commit_sha: Some(commit_sha.to_string()),
            skills: contents.roots,
        },
    );
    write_installed_skill_packages(target_root, &installed)
}

fn skill_install_contents(
    files: &[ExtensionFile],
) -> Result<SkillInstallContents<'_>, ClientError> {
    let mut contents = SkillInstallContents {
        files: Vec::new(),
        roots: BTreeSet::new(),
        removed_roots: BTreeSet::new(),
    };
    for source in files {
        let relative = source.path.strip_prefix(SKILLS_PREFIX).ok_or_else(|| {
            ClientError::new("invalid_response", "skill install bundle is invalid")
        })?;
        let mut parts = relative.split('/');
        let root = parts
            .next()
            .filter(|part| !part.is_empty())
            .ok_or_else(|| {
                ClientError::new("invalid_response", "skill install bundle is invalid")
            })?;
        let remaining = parts.collect::<Vec<_>>();
        if let Some(removed_root) = root.strip_prefix('.') {
            if removed_root.is_empty() || remaining.as_slice() != [".gitkeep"] {
                return Err(ClientError::new(
                    "invalid_response",
                    "skill delete marker is invalid",
                ));
            }
            contents.removed_roots.insert(removed_root.to_string());
            continue;
        }
        contents.roots.insert(root.to_string());
        contents
            .files
            .push((source, decode_extension_file(source)?));
    }
    Ok(contents)
}

pub(crate) fn read_installed_skill_packages(
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

pub(crate) fn retain_listed_skill_packages(
    target_roots: &[PathBuf],
    listed_packages: &BTreeSet<String>,
) -> Result<(), ClientError> {
    for target_root in target_roots {
        let mut installed = read_installed_skill_packages(target_root)?;
        let initial_len = installed.0.len();
        installed
            .0
            .retain(|package, _| listed_packages.contains(package));
        if installed.0.len() != initial_len {
            write_installed_skill_packages(target_root, &installed)?;
        }
    }
    Ok(())
}

pub(crate) struct SkillInstallationStatus {
    pub action: ExtensionInstallAction,
    pub clients: Vec<AgentClient>,
}

pub(crate) fn skill_installation_status(
    targets: &[(AgentClient, PathBuf)],
    package: &str,
    version: &str,
    commit_sha: &str,
) -> Result<SkillInstallationStatus, ClientError> {
    let mut states = BTreeMap::new();
    for (_, root) in targets {
        if !states.contains_key(root) {
            states.insert(root.clone(), read_installed_skill_packages(root)?);
        }
    }
    let mut clients = Vec::new();
    let mut missing = false;
    let mut outdated = false;
    for (client, root) in targets {
        let packages = states.get(root).expect("skill state was loaded");
        let Some(current) = packages.0.get(package) else {
            missing = true;
            continue;
        };
        clients.push(*client);
        if current.version.as_deref() != Some(version)
            || current.commit_sha.as_deref() != Some(commit_sha)
        {
            outdated = true;
        }
    }
    let action = if outdated {
        ExtensionInstallAction::Update
    } else if !clients.is_empty() && missing {
        ExtensionInstallAction::Partial
    } else if clients.is_empty() {
        ExtensionInstallAction::Install
    } else {
        ExtensionInstallAction::Installed
    };
    Ok(SkillInstallationStatus { action, clients })
}

pub(crate) fn outdated_skill_package_targets(
    target_roots: &[PathBuf],
    packages: &[ExtensionPackage],
) -> Result<BTreeMap<String, Vec<PathBuf>>, ClientError> {
    let catalog = packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut targets = BTreeMap::new();
    for target_root in target_roots {
        let installed = read_installed_skill_packages(target_root)?;
        for (name, package) in installed.0 {
            let Some(current) = catalog.get(name.as_str()) else {
                continue;
            };
            if package.version.as_deref() != Some(&current.version)
                || package.commit_sha.as_deref() != Some(&current.commit_sha)
            {
                targets
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .push(target_root.clone());
            }
        }
    }
    Ok(targets)
}

pub(crate) fn skill_installation_metadata(target_root: &Path) -> BTreeMap<String, Option<String>> {
    let Ok(installed) = read_installed_skill_packages(target_root) else {
        return BTreeMap::new();
    };
    installed
        .0
        .into_values()
        .flat_map(|package| {
            package
                .skills
                .into_iter()
                .map(move |skill| (skill, package.version.clone()))
        })
        .collect()
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
        installed.0.insert(
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

fn write_installed_skill_packages(
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

fn skill_package_state_path(target_root: &Path) -> Option<PathBuf> {
    target_root.parent().map(|parent| {
        parent
            .join(PRELAY_STATE_DIRECTORY)
            .join(SKILL_PACKAGE_STATE_FILE)
    })
}

impl InstalledSkillPackages {
    fn release_roots_from_other_packages(&mut self, package: &str, roots: &BTreeSet<String>) {
        self.0.retain(|name, installed| {
            if name != package {
                installed.skills.retain(|skill| !roots.contains(skill));
            }
            !installed.skills.is_empty()
        });
    }
}

fn ensure_skill_roots_available(
    target_root: &Path,
    package: &str,
    roots: &BTreeSet<String>,
    installed: &InstalledSkillPackages,
) -> Result<(), ClientError> {
    for (name, entry) in &installed.0 {
        if name != package && !entry.skills.is_disjoint(roots) {
            return Err(ClientError::new(
                "extension_target_exists",
                "技能目录已由另一个扩展包管理。",
            ));
        }
    }

    for root in roots {
        if target_root.join(root).exists()
            && installed
                .0
                .get(package)
                .is_none_or(|installed| !installed.skills.contains(root))
        {
            return Err(ClientError::new(
                "extension_target_exists",
                "技能目录已存在，确认后可覆盖安装。",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "skills_tests.rs"]
mod tests;
