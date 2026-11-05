use std::fs;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::{
    ExtensionArtifact, ExtensionFile, ExtensionInstallBundle, ExtensionKind, ExtensionVersion,
};
use serde_json::json;
use tempfile::tempdir;

use crate::agents::AgentClient;

use super::{
    host_state::{ExtensionHost, HostState},
    install::install_prepared_bundle,
    uninstall::uninstall_plugin_for_client,
};
use crate::agents::AgentItemKind;

mod hook_identity;

fn bundle(name: &str, artifacts: Vec<ExtensionArtifact>) -> ExtensionInstallBundle {
    bundle_with_version(name, "v1.0.0", artifacts)
}

fn bundle_with_version(
    name: &str,
    tag: &str,
    artifacts: Vec<ExtensionArtifact>,
) -> ExtensionInstallBundle {
    ExtensionInstallBundle {
        name: name.to_string(),
        version: ExtensionVersion {
            tag: tag.to_string(),
            commit_sha: "abc".to_string(),
            updated_at: "2026-11-05T00:00:00Z".to_string(),
        },
        artifacts,
        skipped: Vec::new(),
    }
}

fn file(path: &str, content: &str) -> ExtensionFile {
    ExtensionFile {
        path: path.to_string(),
        content_base64: BASE64.encode(content),
    }
}

#[test]
fn installs_and_uninstalls_a_skill_artifact() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let demo = bundle(
        "demo",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Skill,
            files: vec![file("skills/foo/SKILL.md", "# Foo\n")],
        }],
    );

    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap();
    assert!(home
        .join(".agents")
        .join("skills")
        .join("foo")
        .join("SKILL.md")
        .is_file());
    let state = HostState::load(home, ExtensionHost::Codex).unwrap();
    assert_eq!(state.plugins.len(), 1);
    assert_eq!(
        state.plugins[0].state,
        super::host_state::PluginState::Installed
    );

    uninstall_plugin_for_client(home, AgentClient::ChatGpt, "demo").unwrap();
    assert!(!home.join(".agents").join("skills").join("foo").exists());
    let state = HostState::load(home, ExtensionHost::Codex).unwrap();
    assert!(state.plugins.is_empty());
}

#[test]
fn installs_and_uninstalls_a_hook_artifact() {
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
    let hooks: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".codex").join("hooks.json")).unwrap()).unwrap();
    assert_eq!(
        hooks["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "demo-hook"
    );
    assert!(home
        .join(".prelay")
        .join("packages")
        .join("demo")
        .join("abc")
        .join("hooks")
        .join("hooks.json")
        .is_file());

    uninstall_plugin_for_client(home, AgentClient::CodexCli, "demo").unwrap();
    let hooks: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".codex").join("hooks.json")).unwrap()).unwrap();
    assert!(hooks["hooks"]["SessionStart"]
        .as_array()
        .unwrap()
        .iter()
        .all(|group| group["hooks"].as_array().unwrap().is_empty()));
    assert!(!home
        .join(".prelay")
        .join("packages")
        .join("demo")
        .join("abc")
        .exists());
}

#[test]
fn installs_a_rule_bundle_and_records_its_hash() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let rule = bundle(
        "development-rules",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Rule,
            files: vec![file("AGENTS.md", "# Rules\n")],
        }],
    );

    install_prepared_bundle(home, &rule, &[AgentClient::CodexCli], false).unwrap();
    assert_eq!(
        fs::read_to_string(home.join(".codex").join("AGENTS.md")).unwrap(),
        "# Rules\n"
    );
    let state = HostState::load(home, ExtensionHost::Codex).unwrap();
    let entry = state
        .rule_entry(&home.join(".codex").join("AGENTS.md"))
        .unwrap();
    assert_eq!(entry.package, "development-rules");
    assert_eq!(entry.state, super::host_state::PluginState::Installed);
}

#[test]
fn installs_and_uninstalls_an_mcp_artifact() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let manifest = json!({
        "name": "filesystem",
        "transport": {
            "type": "stdio",
            "command": ["uvx", "mcp-server-filesystem"],
            "cwd": null,
            "environment": {},
            "enabled": true,
            "timeoutMs": null
        }
    })
    .to_string();
    let demo = bundle(
        "demo",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Mcp,
            files: vec![file("server.json", &manifest)],
        }],
    );

    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap();
    let config = fs::read_to_string(home.join(".codex").join("config.toml")).unwrap();
    assert!(config.contains("[mcp_servers.filesystem]"));
    let state = HostState::load(home, ExtensionHost::Codex).unwrap();
    assert!(state.plugins[0].artifacts.iter().any(|artifact| matches!(
        artifact,
        super::host_state::PluginArtifact::Mcp { server_name, .. } if server_name == "filesystem"
    )));

    uninstall_plugin_for_client(home, AgentClient::CodexCli, "demo").unwrap();
    let config = fs::read_to_string(home.join(".codex").join("config.toml")).unwrap();
    assert!(!config.contains("[mcp_servers.filesystem]"));
}

#[test]
fn scan_hides_managed_items_and_shows_one_plugin_row() {
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
        vec![
            ExtensionArtifact {
                kind: ExtensionKind::Skill,
                files: vec![file("skills/foo/SKILL.md", "# Foo\n")],
            },
            ExtensionArtifact {
                kind: ExtensionKind::Hook,
                files: vec![file("hooks/hooks.json", &manifest)],
            },
        ],
    );
    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap();

    let items = crate::agents::scan_agent_items(home, AgentClient::CodexCli).items;
    assert_eq!(
        items
            .iter()
            .filter(|item| item.kind == AgentItemKind::Plugin)
            .count(),
        1
    );
    assert!(!items.iter().any(|item| item.kind == AgentItemKind::Skill));
    assert!(!items.iter().any(|item| item.kind == AgentItemKind::Hook));
}

#[test]
fn refuses_to_replace_an_unmanaged_skill_without_overwrite() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let skill_path = home.join(".agents").join("skills").join("foo");
    fs::create_dir_all(&skill_path).unwrap();
    fs::write(skill_path.join("SKILL.md"), "# Personal\n").unwrap();
    let demo = bundle(
        "demo",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Skill,
            files: vec![file("skills/foo/SKILL.md", "# Managed\n")],
        }],
    );

    let error = install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap_err();
    assert_eq!(error.code(), "extension_target_exists");

    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], true).unwrap();
    assert_eq!(
        fs::read_to_string(skill_path.join("SKILL.md")).unwrap(),
        "# Managed\n"
    );
}

#[test]
fn refuses_to_uninstall_an_mcp_modified_by_the_user() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let manifest = json!({
        "name": "filesystem",
        "transport": {
            "type": "stdio",
            "command": ["uvx", "mcp-server-filesystem"],
            "cwd": null,
            "environment": {},
            "enabled": true,
            "timeoutMs": null
        }
    })
    .to_string();
    let demo = bundle(
        "demo",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Mcp,
            files: vec![file("server.json", &manifest)],
        }],
    );
    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap();
    let config_path = home.join(".codex").join("config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    fs::write(&config_path, config.replace("uvx", "other")).unwrap();

    let error = uninstall_plugin_for_client(home, AgentClient::CodexCli, "demo").unwrap_err();
    assert_eq!(error.code(), "local_extensions_conflict");
    assert!(HostState::load(home, ExtensionHost::Codex)
        .unwrap()
        .plugin("demo")
        .is_some());
    assert!(fs::read_to_string(&config_path)
        .unwrap()
        .contains("[mcp_servers.filesystem]"));
}

#[test]
fn refuses_to_replace_an_unmanaged_rule_without_overwrite() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let target = home.join(".codex").join("AGENTS.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "# Personal\n").unwrap();
    let rule = bundle(
        "development-rules",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Rule,
            files: vec![file("AGENTS.md", "# Managed\n")],
        }],
    );

    let error = install_prepared_bundle(home, &rule, &[AgentClient::CodexCli], false).unwrap_err();
    assert_eq!(error.code(), "extension_target_exists");
    assert_eq!(fs::read_to_string(&target).unwrap(), "# Personal\n");

    install_prepared_bundle(home, &rule, &[AgentClient::CodexCli], true).unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "# Managed\n");
}

#[test]
fn refuses_to_claim_an_unmanaged_hook_without_overwrite() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let hooks_path = home.join(".codex").join("hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    let manifest = json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "demo-hook"}]
            }]
        }
    })
    .to_string();
    fs::write(&hooks_path, &manifest).unwrap();
    let demo = bundle(
        "demo",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Hook,
            files: vec![file("hooks/hooks.json", &manifest)],
        }],
    );

    let error = install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], false).unwrap_err();
    assert_eq!(error.code(), "extension_target_exists");
    assert!(HostState::load(home, ExtensionHost::Codex)
        .unwrap()
        .plugin("demo")
        .is_none());

    install_prepared_bundle(home, &demo, &[AgentClient::CodexCli], true).unwrap();
    assert!(HostState::load(home, ExtensionHost::Codex)
        .unwrap()
        .plugin("demo")
        .is_some());
}

#[test]
fn update_removes_artifacts_that_are_no_longer_in_the_plugin() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let mcp_manifest = json!({
        "name": "filesystem",
        "transport": {
            "type": "stdio",
            "command": ["uvx", "mcp-server-filesystem"],
            "cwd": null,
            "environment": {},
            "enabled": true,
            "timeoutMs": null
        }
    })
    .to_string();
    let hook_manifest = json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "demo-hook"}]
            }]
        }
    })
    .to_string();
    let first = bundle(
        "demo",
        vec![
            ExtensionArtifact {
                kind: ExtensionKind::Skill,
                files: vec![file("skills/foo/SKILL.md", "# Foo\n")],
            },
            ExtensionArtifact {
                kind: ExtensionKind::Mcp,
                files: vec![file("server.json", &mcp_manifest)],
            },
            ExtensionArtifact {
                kind: ExtensionKind::Hook,
                files: vec![file("hooks/hooks.json", &hook_manifest)],
            },
        ],
    );
    install_prepared_bundle(home, &first, &[AgentClient::CodexCli], false).unwrap();

    let second = bundle_with_version(
        "demo",
        "v2.0.0",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Skill,
            files: vec![file("skills/bar/SKILL.md", "# Bar\n")],
        }],
    );
    install_prepared_bundle(home, &second, &[AgentClient::CodexCli], false).unwrap();

    assert!(!home.join(".agents").join("skills").join("foo").exists());
    assert!(home
        .join(".agents")
        .join("skills")
        .join("bar")
        .join("SKILL.md")
        .is_file());
    assert!(!fs::read_to_string(home.join(".codex").join("config.toml"))
        .unwrap()
        .contains("[mcp_servers.filesystem]"));
    let hooks: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".codex").join("hooks.json")).unwrap()).unwrap();
    assert!(hooks["hooks"]["SessionStart"]
        .as_array()
        .unwrap()
        .iter()
        .all(|group| group["hooks"].as_array().unwrap().is_empty()));
    assert!(!home
        .join(".prelay")
        .join("packages")
        .join("demo")
        .join("abc")
        .exists());
}
