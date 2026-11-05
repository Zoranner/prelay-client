use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::extensions::{
    host_state::HostState,
    skills::{managed_skill_roots, ManagedSkillRoot},
};

use super::super::model::{AgentItem, AgentItemKind, AgentItemSource};

pub(super) fn scan_skills(root: PathBuf, state: &HostState) -> Vec<AgentItem> {
    let managed = managed_skill_roots(state, &root);
    let mut items = Vec::new();
    let mut packages = BTreeMap::new();
    visit_skill_directory(&root, &root, &managed, &mut packages, &mut items);
    items.extend(
        packages
            .into_iter()
            .map(|(package, (version, mut members))| {
                members.sort();
                AgentItem {
                    kind: AgentItemKind::Skill,
                    name: package.clone(),
                    version,
                    package: Some(package),
                    hook_hash: None,
                    matcher: None,
                    command: None,
                    transport: None,
                    target: None,
                    description: None,
                    file_count: None,
                    members,
                    artifacts: Vec::new(),
                    source: AgentItemSource::Team,
                    source_path: root.display().to_string(),
                    error_message: None,
                }
            }),
    );
    items
}

fn visit_skill_directory(
    base: &Path,
    path: &Path,
    managed: &BTreeMap<String, ManagedSkillRoot>,
    packages: &mut BTreeMap<String, (Option<String>, Vec<String>)>,
    items: &mut Vec<AgentItem>,
) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                match managed_skill_owner(base, &path, managed) {
                    Some(owner) => {
                        let entry = packages
                            .entry(owner.package.clone())
                            .or_insert_with(|| (Some(owner.version.clone()), Vec::new()));
                        entry.1.push(name);
                    }
                    None => items.push(AgentItem {
                        kind: AgentItemKind::Skill,
                        name,
                        version: None,
                        package: None,
                        hook_hash: None,
                        matcher: None,
                        command: None,
                        transport: None,
                        target: None,
                        description: skill_description(&skill_file),
                        file_count: file_count(&path),
                        members: Vec::new(),
                        artifacts: Vec::new(),
                        source: AgentItemSource::Personal,
                        source_path: path.display().to_string(),
                        error_message: None,
                    }),
                }
            }
            visit_skill_directory(base, &path, managed, packages, items);
        }
    }
}

fn skill_description(skill_file: &Path) -> Option<String> {
    let contents = fs::read_to_string(skill_file).ok()?;
    let mut lines = contents.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        let Some(value) = line.strip_prefix("description:") else {
            continue;
        };
        let value = value.trim().trim_matches(['"', '\'']).trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn file_count(path: &Path) -> Option<u32> {
    fn count(path: &Path) -> usize {
        let Ok(entries) = fs::read_dir(path) else {
            return 0;
        };
        entries
            .flatten()
            .map(|entry| match entry.file_type() {
                Ok(kind) if kind.is_file() => 1,
                Ok(kind) if kind.is_dir() => count(&entry.path()),
                _ => 0,
            })
            .sum()
    }
    u32::try_from(count(path)).ok()
}

fn managed_skill_owner<'a>(
    base: &Path,
    path: &Path,
    managed: &'a BTreeMap<String, ManagedSkillRoot>,
) -> Option<&'a ManagedSkillRoot> {
    let relative = path.strip_prefix(base).ok()?;
    let root = relative.components().next()?;
    managed.get(root.as_os_str().to_string_lossy().as_ref())
}
