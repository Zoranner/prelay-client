use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use crate::{
    agents::{AgentArtifactGroup, AgentItem, AgentItemKind, AgentItemSource},
    extensions::{
        host_state::{PluginArtifact, PluginRecord},
        mcp::ManagedMcpServer,
    },
};

pub(super) fn plugin_artifact_groups(plugin: &PluginRecord) -> Vec<AgentArtifactGroup> {
    let mut skills = BTreeSet::new();
    let mut mcps = BTreeSet::new();
    let mut hooks = BTreeSet::new();
    for artifact in &plugin.artifacts {
        match artifact {
            PluginArtifact::Skill { names, .. } => {
                skills.extend(names.iter().filter(|name| !name.starts_with('.')).cloned())
            }
            PluginArtifact::Mcp { server_name, .. } => {
                mcps.insert(server_name.clone());
            }
            PluginArtifact::Hook { event, matcher, .. } => {
                hooks.insert(match matcher {
                    Some(matcher) => format!("{event} · {matcher}"),
                    None => event.clone(),
                });
            }
        }
    }
    [
        (AgentItemKind::Skill, skills),
        (AgentItemKind::Mcp, mcps),
        (AgentItemKind::Hook, hooks),
    ]
    .into_iter()
    .filter_map(|(kind, items)| {
        (!items.is_empty()).then(|| AgentArtifactGroup {
            kind,
            items: items.into_iter().collect(),
        })
    })
    .collect()
}

pub(super) fn read_toml(path: &Path) -> Result<Option<toml::Value>, ()> {
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| toml::from_str::<toml::Value>(&contents).ok())
        .ok_or(())
        .map(Some)
}

pub(super) fn toml_items(
    value: &toml::Value,
    section: &str,
    kind: AgentItemKind,
    path: &Path,
    managed: &BTreeMap<String, ManagedMcpServer>,
) -> Vec<AgentItem> {
    let Some(entries) = value.get(section).and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|(name, entry)| {
            if entry
                .get("enabled")
                .and_then(toml::Value::as_bool)
                .is_some_and(|enabled| !enabled)
            {
                return None;
            }
            let managed_entry = managed.get(name);
            let (transport, target) = mcp_details(entry);
            Some(AgentItem {
                kind,
                name: name.to_owned(),
                version: managed_entry.map(|managed| managed.version.clone()),
                package: managed_entry.map(|managed| managed.package.clone()),
                hook_hash: None,
                matcher: None,
                command: None,
                transport,
                target,
                description: None,
                file_count: None,
                members: Vec::new(),
                artifacts: Vec::new(),
                source: if managed_entry.is_some() {
                    AgentItemSource::Team
                } else {
                    AgentItemSource::Personal
                },
                source_path: path.display().to_string(),
                error_message: None,
            })
        })
        .collect()
}

fn mcp_details(entry: &toml::Value) -> (Option<String>, Option<String>) {
    if let Some(url) = entry.get("url").and_then(toml::Value::as_str) {
        return (Some("http".to_string()), Some(url.to_string()));
    }
    let Some(command) = entry.get("command").and_then(toml::Value::as_str) else {
        return (None, None);
    };
    let mut parts = vec![command.to_string()];
    if let Some(args) = entry.get("args").and_then(toml::Value::as_array) {
        parts.extend(
            args.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string),
        );
    }
    (Some("stdio".to_string()), Some(parts.join(" ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::host_state::PluginState;

    #[test]
    fn plugin_artifact_groups_hide_dot_prefixed_skills() {
        let plugin = PluginRecord {
            schema_version: 2,
            name: "engineering".to_string(),
            version: "v0.3.0".to_string(),
            commit_sha: "abc".to_string(),
            state: PluginState::Installed,
            artifacts: vec![PluginArtifact::Skill {
                target: "C:\\Users\\u\\.agents\\skills".to_string(),
                names: vec![
                    ".review-dotnet".to_string(),
                    ".review-nuxt".to_string(),
                    "review-engineering".to_string(),
                ],
            }],
        };

        let groups = plugin_artifact_groups(&plugin);
        let skills = groups
            .iter()
            .find(|group| group.kind == AgentItemKind::Skill)
            .expect("skill group");

        assert_eq!(skills.items, vec!["review-engineering"]);
    }
}
