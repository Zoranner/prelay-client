use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::{PluginRecord, PluginState, StateError};

pub(super) fn detect_conflicts(
    plugins: &[PluginRecord],
    directory: &Path,
) -> (BTreeSet<String>, Vec<StateError>) {
    let mut owners = BTreeMap::<String, Vec<(String, PluginState)>>::new();
    for plugin in plugins {
        for identity in plugin
            .artifacts
            .iter()
            .flat_map(super::support::artifact_identities)
        {
            owners
                .entry(identity)
                .or_default()
                .push((plugin.name.clone(), plugin.state));
        }
    }
    let mut conflicts = BTreeSet::new();
    let mut errors = Vec::new();
    for (identity, owners) in owners {
        let installed = owners
            .iter()
            .filter(|(_, state)| *state == PluginState::Installed)
            .map(|(name, _)| name.clone())
            .collect::<BTreeSet<_>>();
        if installed.is_empty() {
            continue;
        }
        let installed_count = owners
            .iter()
            .filter(|(_, state)| *state == PluginState::Installed)
            .count();
        if installed.len() == 1 && installed_count == 1 {
            continue;
        }
        conflicts.extend(installed.iter().cloned());
        let message = if installed.len() == 1 {
            format!("扩展制品 {identity} 在同一个插件中重复声明。")
        } else {
            format!(
                "扩展制品 {identity} 同时由多个插件声明：{}。",
                installed.into_iter().collect::<Vec<_>>().join("、")
            )
        };
        errors.push(StateError {
            path: directory.display().to_string(),
            message,
            readonly: false,
        });
    }
    (conflicts, errors)
}
