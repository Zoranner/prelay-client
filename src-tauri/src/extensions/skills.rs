use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionFile;
use serde::{Deserialize, Serialize};

use crate::relay::client::ClientError;

use super::{atomic_write, storage_error, SKILLS_PREFIX};

const MANAGED_SKILLS_DIRECTORY: &str = ".prelay/skills";

#[derive(Debug, Deserialize, Serialize)]
struct ManagedSkillPackage {
    roots: BTreeSet<String>,
}

pub(super) fn install_skill_files(
    target_root: &Path,
    package: &str,
    files: &[ExtensionFile],
) -> Result<(), ClientError> {
    let roots = skill_roots(files)?;
    let manifest_path = managed_skill_manifest_path(target_root, package)?;
    let previous = read_managed_skill_package(&manifest_path)?;

    ensure_skill_roots_available(target_root, package, &roots, previous.as_ref())?;

    let mut roots_to_replace = roots.clone();
    if let Some(previous) = &previous {
        roots_to_replace.extend(previous.roots.iter().cloned());
    }
    for root in roots_to_replace {
        let path = target_root.join(root);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(storage_error)?;
        }
    }

    for source in files {
        let relative = source
            .path
            .strip_prefix(SKILLS_PREFIX)
            .expect("validated skill path");
        atomic_write(&target_root.join(relative), source.content.as_bytes())?;
    }
    write_managed_skill_package(&manifest_path, &ManagedSkillPackage { roots })
}

fn skill_roots(files: &[ExtensionFile]) -> Result<BTreeSet<String>, ClientError> {
    files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(SKILLS_PREFIX)
                .and_then(|path| path.split('/').next())
                .filter(|root| !root.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| {
                    ClientError::new("invalid_response", "skill install bundle is invalid")
                })
        })
        .collect()
}

fn managed_skill_manifest_path(target_root: &Path, package: &str) -> Result<PathBuf, ClientError> {
    if package.is_empty() {
        return Err(ClientError::new(
            "invalid_response",
            "skill package name is empty",
        ));
    }
    let key = package
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let parent = target_root.parent().ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "skill target root has no parent directory",
        )
    })?;
    Ok(parent
        .join(MANAGED_SKILLS_DIRECTORY)
        .join(format!("{key}.json")))
}

fn read_managed_skill_package(path: &Path) -> Result<Option<ManagedSkillPackage>, ClientError> {
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents)
            .map(Some)
            .map_err(|error| {
                ClientError::new(
                    "local_extensions_error",
                    format!("无法读取已安装 Skill 清单：{error}"),
                )
            }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(storage_error(error)),
    }
}

fn write_managed_skill_package(
    path: &Path,
    package: &ManagedSkillPackage,
) -> Result<(), ClientError> {
    let contents = serde_json::to_vec(package).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存已安装 Skill 清单：{error}"),
        )
    })?;
    atomic_write(path, &contents)
}

fn ensure_skill_roots_available(
    target_root: &Path,
    package: &str,
    roots: &BTreeSet<String>,
    previous: Option<&ManagedSkillPackage>,
) -> Result<(), ClientError> {
    let managed_directory = target_root
        .parent()
        .ok_or_else(|| {
            ClientError::new(
                "local_extensions_error",
                "skill target root has no parent directory",
            )
        })?
        .join(MANAGED_SKILLS_DIRECTORY);
    if managed_directory.exists() {
        for entry in fs::read_dir(&managed_directory).map_err(storage_error)? {
            let path = entry.map_err(storage_error)?.path();
            if path.extension().is_none_or(|extension| extension != "json") {
                continue;
            }
            let Some(managed) = read_managed_skill_package(&path)? else {
                continue;
            };
            if path != managed_skill_manifest_path(target_root, package)?
                && !managed.roots.is_disjoint(roots)
            {
                return Err(ClientError::new(
                    "local_extensions_error",
                    "技能目录已由另一个扩展包管理。",
                ));
            }
        }
    }

    for root in roots {
        if target_root.join(root).exists()
            && previous.is_none_or(|managed| !managed.roots.contains(root))
        {
            return Err(ClientError::new(
                "local_extensions_error",
                "技能目录已存在且不受当前扩展包管理。",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use prelay_protocol::ExtensionFile;
    use tempfile::tempdir;

    use super::install_skill_files;

    fn skill_file(path: &str, content: &str) -> ExtensionFile {
        ExtensionFile {
            path: path.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn reinstalling_a_skill_package_removes_stale_files_and_directories() {
        let directory = tempdir().unwrap();
        let root = directory.path().join("skills");

        install_skill_files(
            &root,
            "engineering",
            &[
                skill_file("skills/check/SKILL.md", "old"),
                skill_file("skills/retired/SKILL.md", "retired"),
            ],
        )
        .unwrap();
        fs::write(root.join("check").join("stale.md"), "stale").unwrap();

        install_skill_files(
            &root,
            "engineering",
            &[skill_file("skills/check/SKILL.md", "new")],
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(root.join("check").join("SKILL.md")).unwrap(),
            "new"
        );
        assert!(!root.join("check").join("stale.md").exists());
        assert!(!root.join("retired").exists());
    }

    #[test]
    fn installing_a_skill_does_not_replace_another_packages_directory() {
        let directory = tempdir().unwrap();
        let root = directory.path().join("skills");
        let files = [skill_file("skills/shared/SKILL.md", "first")];

        install_skill_files(&root, "first-package", &files).unwrap();
        let result = install_skill_files(&root, "second-package", &files);

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(root.join("shared").join("SKILL.md")).unwrap(),
            "first"
        );
    }

    #[test]
    fn installing_a_skill_does_not_replace_an_unmanaged_directory() {
        let directory = tempdir().unwrap();
        let root = directory.path().join("skills");
        fs::create_dir_all(root.join("manual")).unwrap();
        fs::write(root.join("manual").join("SKILL.md"), "manual").unwrap();

        let result = install_skill_files(
            &root,
            "managed-package",
            &[skill_file("skills/manual/SKILL.md", "managed")],
        );

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(root.join("manual").join("SKILL.md")).unwrap(),
            "manual"
        );
    }
}
