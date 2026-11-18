use std::{collections::BTreeMap, fs, path::Path};

use prelay_protocol::{ExtensionArtifact, ExtensionKind};
use serde_json::json;
use tempfile::tempdir;

use crate::agents::AgentClient;

use super::super::{
    host_state::{ExtensionHost, HostState},
    install::install_prepared_bundle,
    uninstall::uninstall_plugin_for_client,
};
use super::{bundle, bundle_with_identity, file};

fn trust_entries(home: &Path) -> BTreeMap<String, String> {
    let path = home.join(".codex").join("config.toml");
    let Ok(contents) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    let document: toml::Value = toml::from_str(&contents).unwrap();
    document
        .get("hooks")
        .and_then(|hooks| hooks.get("state"))
        .and_then(toml::Value::as_table)
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| {
            value
                .get("trusted_hash")
                .and_then(toml::Value::as_str)
                .map(|hash| (key.clone(), hash.to_string()))
        })
        .collect()
}

#[test]
fn trusts_only_prelay_managed_hooks_and_removes_trust_on_uninstall() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let hooks_path = home.join(".codex").join("hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    fs::write(
        &hooks_path,
        serde_json::to_vec(&json!({
            "hooks": {
                "SessionStart": [{
                    "hooks": [{"type": "command", "command": "user-hook"}]
                }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let user_key = format!("{}:session_start:0:0", hooks_path.display());
    fs::write(
        home.join(".codex").join("config.toml"),
        format!("[hooks.state.'{user_key}']\ntrusted_hash = \"sha256:user\"\n"),
    )
    .unwrap();
    let manifest = json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "demo-hook"}]
            }]
        }
    })
    .to_string();
    let demo = bundle(
        "demo",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Hook,
            files: vec![file("hooks/hooks.json", &manifest)],
        }],
    );

    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap();
    let managed_key = format!("{}:session_start:0:1", hooks_path.display());
    let entries = trust_entries(home);
    assert_eq!(entries.get(&user_key), Some(&"sha256:user".to_string()));
    assert!(entries
        .get(&managed_key)
        .is_some_and(|hash| hash.starts_with("sha256:") && hash != "sha256:user"));

    uninstall_plugin_for_client(home, AgentClient::CodexCli, "demo").unwrap();
    let entries = trust_entries(home);
    assert_eq!(entries.get(&user_key), Some(&"sha256:user".to_string()));
    assert!(!entries.contains_key(&managed_key));
}

#[test]
fn rewrites_prelay_trust_when_uninstalling_moves_another_hook_index() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let first_manifest = json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "first-hook"}]
            }]
        }
    })
    .to_string();
    let second_manifest = json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "second-hook"}]
            }]
        }
    })
    .to_string();
    let first = bundle_with_identity(
        "first",
        "v1.0.0",
        "first-commit",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Hook,
            files: vec![file("hooks/hooks.json", &first_manifest)],
        }],
    );
    let second = bundle_with_identity(
        "second",
        "v1.0.0",
        "second-commit",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Hook,
            files: vec![file("hooks/hooks.json", &second_manifest)],
        }],
    );

    install_prepared_bundle(home, &first, &[AgentClient::CodexCli], false).unwrap();
    install_prepared_bundle(home, &second, &[AgentClient::CodexCli], false).unwrap();
    let hooks_path = home.join(".codex").join("hooks.json");
    let first_key = format!("{}:session_start:0:0", hooks_path.display());
    let second_old_key = format!("{}:session_start:0:1", hooks_path.display());
    assert!(trust_entries(home).contains_key(&first_key));
    assert!(trust_entries(home).contains_key(&second_old_key));

    uninstall_plugin_for_client(home, AgentClient::CodexCli, "first").unwrap();
    let second_new_key = format!("{}:session_start:0:0", hooks_path.display());
    let entries = trust_entries(home);
    assert!(!entries.contains_key(&second_old_key));
    assert!(entries.contains_key(&second_new_key));

    let state = HostState::load(home, ExtensionHost::Codex).unwrap();
    assert_eq!(
        entries,
        super::super::hooks::codex_trust_entries(home, &state).unwrap()
    );
}

#[test]
fn preserves_a_user_disabled_prelay_hook_when_refreshing_trust() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let manifest = json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "demo-hook"}]
            }]
        }
    })
    .to_string();
    let demo = bundle(
        "demo",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Hook,
            files: vec![file("hooks/hooks.json", &manifest)],
        }],
    );
    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap();

    let hooks_path = home.join(".codex").join("hooks.json");
    let key = format!("{}:session_start:0:0", hooks_path.display());
    let config_path = home.join(".codex").join("config.toml");
    let mut document = fs::read_to_string(&config_path)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    document["hooks"]["state"][key.as_str()]["enabled"] = toml_edit::value(false);
    fs::write(&config_path, document.to_string()).unwrap();

    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap();
    let document: toml::Value = toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(
        document["hooks"]["state"][key.as_str()]["enabled"].as_bool(),
        Some(false)
    );
}
