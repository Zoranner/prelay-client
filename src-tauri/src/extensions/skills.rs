use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use prelay_protocol::ExtensionFile;

use crate::relay::client::ClientError;

use super::{
    atomic_write, decode_extension_file,
    host_state::{normalize_path, ArtifactHealth, HostState, PluginArtifact},
    storage_error, SKILLS_PREFIX,
};

#[derive(Clone, Debug)]
pub(crate) struct ManagedSkillRoot {
    pub package: String,
    pub version: String,
}

struct SkillInstallContents<'a> {
    files: Vec<(&'a ExtensionFile, Vec<u8>)>,
    roots: BTreeSet<String>,
    removed_roots: BTreeSet<String>,
}

pub(crate) fn preflight_skill_files(
    target_root: &Path,
    package: &str,
    files: &[ExtensionFile],
    overwrite: bool,
    state: &HostState,
) -> Result<Vec<String>, ClientError> {
    let contents = skill_install_contents(files)?;
    ensure_available(target_root, package, &contents.roots, overwrite, state)?;
    Ok(contents.roots.into_iter().collect())
}

pub(super) fn install_skill_files(
    target_root: &Path,
    package: &str,
    files: &[ExtensionFile],
    overwrite: bool,
    state: &HostState,
) -> Result<Vec<String>, ClientError> {
    if package.is_empty() {
        return Err(ClientError::new(
            "invalid_response",
            "skill package name is empty",
        ));
    }
    let contents = skill_install_contents(files)?;
    ensure_available(target_root, package, &contents.roots, overwrite, state)?;
    for root in &contents.roots {
        let path = target_root.join(root);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(storage_error)?;
        }
    }
    for (source, content) in &contents.files {
        let relative = source
            .path
            .strip_prefix(SKILLS_PREFIX)
            .expect("validated skill path");
        atomic_write(&target_root.join(relative), content)?;
    }
    for root in &contents.removed_roots {
        let path = target_root.join(root);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(storage_error)?;
        }
    }
    Ok(contents.roots.into_iter().collect())
}

pub(crate) fn remove_skill_roots(target_root: &Path, names: &[String]) -> Result<(), ClientError> {
    for name in names {
        let Some(root) = safe_skill_root(name) else {
            return Err(ClientError::new(
                "local_extensions_error",
                "已安装 Skill 状态包含无效目录。",
            ));
        };
        let path = target_root.join(root);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(storage_error)?;
        }
    }
    Ok(())
}

pub(crate) fn health(target_root: &Path, names: &[String]) -> ArtifactHealth {
    if names.is_empty() {
        return ArtifactHealth::Missing;
    }
    if names
        .iter()
        .all(|name| safe_skill_root(name).is_some_and(|root| target_root.join(root).is_dir()))
    {
        ArtifactHealth::Managed
    } else {
        ArtifactHealth::Missing
    }
}

pub(crate) fn managed_skill_roots(
    state: &HostState,
    target_root: &Path,
) -> BTreeMap<String, ManagedSkillRoot> {
    let target = normalize_path(target_root);
    let mut roots = BTreeMap::new();
    for plugin in state.active_plugins() {
        for artifact in &plugin.artifacts {
            let PluginArtifact::Skill {
                target: artifact_target,
                names,
            } = artifact
            else {
                continue;
            };
            if artifact_target != &target {
                continue;
            }
            for name in names {
                roots.insert(
                    name.clone(),
                    ManagedSkillRoot {
                        package: plugin.name.clone(),
                        version: plugin.version.clone(),
                    },
                );
            }
        }
    }
    roots
}

fn ensure_available(
    target_root: &Path,
    package: &str,
    roots: &BTreeSet<String>,
    overwrite: bool,
    state: &HostState,
) -> Result<(), ClientError> {
    let target = normalize_path(target_root);
    for plugin in state.active_plugins() {
        if plugin.name == package {
            continue;
        }
        for artifact in &plugin.artifacts {
            if let PluginArtifact::Skill {
                target: artifact_target,
                names,
            } = artifact
            {
                if artifact_target == &target && names.iter().any(|name| roots.contains(name)) {
                    return Err(ClientError::new(
                        "extension_target_exists",
                        "技能目录已由另一个扩展包管理。",
                    ));
                }
            }
        }
    }
    if overwrite {
        return Ok(());
    }
    let owned = state
        .plugin(package)
        .into_iter()
        .flat_map(|plugin| plugin.artifacts.iter())
        .filter_map(|artifact| match artifact {
            PluginArtifact::Skill {
                target: artifact_target,
                names,
            } if artifact_target == &target => Some(names.iter().cloned()),
            _ => None,
        })
        .flatten()
        .collect::<BTreeSet<_>>();
    for root in roots {
        if target_root.join(root).exists() && !owned.contains(root) {
            return Err(ClientError::new(
                "extension_target_exists",
                "技能目录已存在，确认后可覆盖安装。",
            ));
        }
    }
    Ok(())
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

fn safe_skill_root(root: &str) -> Option<&str> {
    (!root.is_empty() && !matches!(root, "." | "..") && !root.contains(['/', '\\'])).then_some(root)
}
