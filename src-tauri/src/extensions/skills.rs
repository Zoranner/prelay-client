use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionFile;

use crate::{agents::AgentClient, relay::client::ClientError};

use super::{
    atomic_write, decode_extension_file,
    skill_state::{
        read_installed_skill_packages, write_installed_skill_packages, InstalledSkillPackage,
        InstalledSkillPackages,
    },
    storage_error, ExtensionInstallAction, SKILLS_PREFIX,
};

#[derive(Debug, Clone)]
pub(crate) struct ManagedSkillRoot {
    pub package: String,
    pub version: Option<String>,
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
    installed.packages.insert(
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

pub(crate) fn retain_listed_skill_packages(
    target_roots: &[PathBuf],
    listed_packages: &BTreeSet<String>,
) -> Result<(), ClientError> {
    for target_root in target_roots {
        let mut installed = read_installed_skill_packages(target_root)?;
        let initial_len = installed.packages.len();
        installed
            .packages
            .retain(|package, _| listed_packages.contains(package));
        if installed.packages.len() != initial_len {
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
    let mut incomplete = false;
    let mut outdated = false;
    let mut abandoned_roots = BTreeSet::new();
    for (client, root) in targets {
        let packages = states.get(root).expect("skill state was loaded");
        let Some(current) = packages.packages.get(package) else {
            missing = true;
            continue;
        };
        let present = current
            .skills
            .iter()
            .filter(|skill| skill_root_exists(root, skill))
            .count();
        if present == 0 {
            missing = true;
            abandoned_roots.insert(root.clone());
            continue;
        }
        clients.push(*client);
        if present < current.skills.len() {
            incomplete = true;
        }
        if current.version.as_deref() != Some(version)
            || current.commit_sha.as_deref() != Some(commit_sha)
        {
            outdated = true;
        }
    }
    for root in abandoned_roots {
        let Some(packages) = states.get_mut(&root) else {
            continue;
        };
        if packages.packages.remove(package).is_some() {
            write_installed_skill_packages(&root, packages)?;
        }
    }
    let action = if outdated {
        ExtensionInstallAction::Update
    } else if incomplete || (!clients.is_empty() && missing) {
        ExtensionInstallAction::Partial
    } else if clients.is_empty() {
        ExtensionInstallAction::Install
    } else {
        ExtensionInstallAction::Installed
    };
    Ok(SkillInstallationStatus { action, clients })
}

pub(crate) fn uninstall_skill_package(
    target_root: &Path,
    package: &str,
) -> Result<(), ClientError> {
    let mut installed = read_installed_skill_packages(target_root)?;
    let Some(entry) = installed.packages.remove(package) else {
        return Err(ClientError::new(
            "local_extensions_error",
            format!("本机没有扩展包 {package} 的 Skill 安装记录。"),
        ));
    };
    for skill in entry.skills {
        let root = safe_skill_root(&skill).ok_or_else(|| {
            ClientError::new("local_extensions_error", "已安装 Skill 状态包含无效目录。")
        })?;
        let path = target_root.join(root);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(storage_error)?;
        }
    }
    write_installed_skill_packages(target_root, &installed)
}

pub(crate) fn managed_skill_roots(target_root: &Path) -> BTreeMap<String, ManagedSkillRoot> {
    let Ok(installed) = read_installed_skill_packages(target_root) else {
        return BTreeMap::new();
    };
    let mut roots = BTreeMap::new();
    for (package, entry) in installed.packages {
        for skill in entry.skills {
            roots.insert(
                skill,
                ManagedSkillRoot {
                    package: package.clone(),
                    version: entry.version.clone(),
                },
            );
        }
    }
    roots
}

fn skill_root_exists(target_root: &Path, root: &str) -> bool {
    safe_skill_root(root).is_some_and(|root| target_root.join(root).is_dir())
}

fn safe_skill_root(root: &str) -> Option<&str> {
    (!root.is_empty() && !matches!(root, "." | "..") && !root.contains(['/', '\\'])).then_some(root)
}

fn ensure_skill_roots_available(
    target_root: &Path,
    package: &str,
    roots: &BTreeSet<String>,
    installed: &InstalledSkillPackages,
) -> Result<(), ClientError> {
    for (name, entry) in &installed.packages {
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
                .packages
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
