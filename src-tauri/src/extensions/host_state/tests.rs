use std::fs;

use serde_json::json;
use tempfile::tempdir;

use super::{
    ensure_migrated_locked, lock, write_plugin, ExtensionHost, HostState, PluginArtifact,
    PluginRecord, PluginState, RuleEntry, PLUGIN_SCHEMA_VERSION, RULE_SCHEMA_VERSION,
};

use crate::agents::AgentClient;

fn write(path: &std::path::Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn writes_and_reads_a_plugin_record() {
    let directory = tempdir().unwrap();
    let host = ExtensionHost::Codex;
    let plugin = PluginRecord {
        schema_version: PLUGIN_SCHEMA_VERSION,
        name: "demo".to_string(),
        version: "v1.0.0".to_string(),
        commit_sha: "abc".to_string(),
        state: PluginState::Installed,
        artifacts: vec![PluginArtifact::Skill {
            target: "C:\\Users\\u\\.agents\\skills".to_string(),
            names: vec!["foo".to_string()],
        }],
    };
    write_plugin(directory.path(), host, &plugin).unwrap();

    let state = HostState::load_locked(directory.path(), host).unwrap();
    assert_eq!(state.plugins.len(), 1);
    assert_eq!(state.plugins[0].name, "demo");
    assert_eq!(state.errors.len(), 0);
}

#[test]
fn rejects_plugin_file_name_that_does_not_match_name() {
    let directory = tempdir().unwrap();
    let host = ExtensionHost::Codex;
    let path = host.plugin_path(directory.path(), "demo");
    write(
        &path,
        &serde_json::to_string(&json!({
            "schemaVersion": PLUGIN_SCHEMA_VERSION,
            "name": "other",
            "version": "v1",
            "commitSha": "abc",
            "state": "installed",
            "artifacts": []
        }))
        .unwrap(),
    );

    let state = HostState::load_locked(directory.path(), host).unwrap();
    assert!(state.plugins.is_empty());
    assert_eq!(state.errors.len(), 1);
}

#[test]
fn rejects_newer_schema_version_as_read_only() {
    let directory = tempdir().unwrap();
    let host = ExtensionHost::Codex;
    let path = host.plugin_path(directory.path(), "demo");
    write(
        &path,
        &serde_json::to_string(&json!({
            "schemaVersion": PLUGIN_SCHEMA_VERSION + 1,
            "name": "demo",
            "version": "v1",
            "commitSha": "abc",
            "state": "installed",
            "artifacts": []
        }))
        .unwrap(),
    );

    let state = HostState::load_locked(directory.path(), host).unwrap();
    assert!(state.readonly);
    assert!(state.ensure_writable().is_err());
}

#[test]
fn rejects_duplicate_artifact_ownership() {
    let directory = tempdir().unwrap();
    let host = ExtensionHost::Codex;
    let artifact = PluginArtifact::Skill {
        target: "C:\\Users\\u\\.agents\\skills".to_string(),
        names: vec!["foo".to_string()],
    };
    for name in ["one", "two"] {
        write_plugin(
            directory.path(),
            host,
            &PluginRecord {
                schema_version: PLUGIN_SCHEMA_VERSION,
                name: name.to_string(),
                version: "v1".to_string(),
                commit_sha: "abc".to_string(),
                state: PluginState::Installed,
                artifacts: vec![artifact.clone()],
            },
        )
        .unwrap();
    }

    let state = HostState::load_locked(directory.path(), host).unwrap();
    assert_eq!(state.plugins.len(), 2);
    assert_eq!(state.errors.len(), 1);
    assert!(state.is_conflicted("one"));
    assert!(state.is_conflicted("two"));
    assert!(state.plugin("one").is_none());
    assert_eq!(state.active_plugins().count(), 0);
    assert!(state.ensure_writable().is_err());
}

#[test]
fn ignores_incomplete_plugins_when_resolving_duplicate_ownership() {
    let directory = tempdir().unwrap();
    let host = ExtensionHost::Codex;
    let artifact = PluginArtifact::Skill {
        target: "C:\\Users\\u\\.agents\\skills".to_string(),
        names: vec!["foo".to_string()],
    };
    write_plugin(
        directory.path(),
        host,
        &PluginRecord {
            schema_version: PLUGIN_SCHEMA_VERSION,
            name: "installed".to_string(),
            version: "v1".to_string(),
            commit_sha: "abc".to_string(),
            state: PluginState::Installed,
            artifacts: vec![artifact.clone()],
        },
    )
    .unwrap();
    write_plugin(
        directory.path(),
        host,
        &PluginRecord {
            schema_version: PLUGIN_SCHEMA_VERSION,
            name: "installing".to_string(),
            version: "v1".to_string(),
            commit_sha: "abc".to_string(),
            state: PluginState::Installing,
            artifacts: vec![artifact],
        },
    )
    .unwrap();

    let state = HostState::load_locked(directory.path(), host).unwrap();
    assert!(state.errors.is_empty());
    assert!(!state.is_conflicted("installed"));
    assert!(!state.is_conflicted("installing"));
    assert_eq!(
        state
            .active_plugins()
            .map(|plugin| plugin.name.as_str())
            .collect::<Vec<_>>(),
        vec!["installed"]
    );
    assert_eq!(state.plugin_names().len(), 2);
    assert!(state.plugin("installing").is_some());
    assert!(state.ensure_writable().is_ok());
}

#[test]
fn uninstalling_a_partial_plugin_does_not_remove_an_installed_plugin_skill() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let host = ExtensionHost::Codex;
    let skill_root = home.join(".agents").join("skills");
    write(&skill_root.join("foo").join("SKILL.md"), "# Foo\n");
    for (name, state) in [
        ("installed", PluginState::Installed),
        ("installing", PluginState::Installing),
    ] {
        write_plugin(
            home,
            host,
            &PluginRecord {
                schema_version: PLUGIN_SCHEMA_VERSION,
                name: name.to_string(),
                version: "v1".to_string(),
                commit_sha: "abc".to_string(),
                state,
                artifacts: vec![PluginArtifact::Skill {
                    target: super::normalize_path(&skill_root),
                    names: vec!["foo".to_string()],
                }],
            },
        )
        .unwrap();
    }

    crate::extensions::uninstall::uninstall_plugin_for_client(
        home,
        AgentClient::CodexCli,
        "installing",
    )
    .unwrap();

    assert!(skill_root.join("foo").join("SKILL.md").is_file());
    let state = HostState::load_locked(home, host).unwrap();
    assert!(state.plugin("installing").is_none());
    assert!(state.plugin("installed").is_some());
}

#[test]
fn rejects_newer_rule_schema_version_as_read_only() {
    let directory = tempdir().unwrap();
    let host = ExtensionHost::Codex;
    write(
        &host.rule_path(directory.path()),
        &serde_json::to_string(&json!({
            "schemaVersion": RULE_SCHEMA_VERSION + 1,
            "targets": {}
        }))
        .unwrap(),
    );

    let state = HostState::load_locked(directory.path(), host).unwrap();
    assert!(state.readonly);
    assert!(state.ensure_writable().is_err());
}

#[test]
fn rule_health_reports_a_user_modification() {
    let directory = tempdir().unwrap();
    let target = directory.path().join(".codex").join("AGENTS.md");
    write(&target, "# Rules\n");
    let entry = RuleEntry {
        package: "development-rules".to_string(),
        version: "v1".to_string(),
        commit_sha: "abc".to_string(),
        content_hash: super::hash_bytes(b"# Rules\n"),
        state: PluginState::Installed,
    };

    assert_eq!(
        crate::extensions::rules::health(&target, &entry),
        super::ArtifactHealth::Managed
    );
    fs::write(&target, "# User edit\n").unwrap();
    assert_eq!(
        crate::extensions::rules::health(&target, &entry),
        super::ArtifactHealth::Modified
    );
}

#[test]
fn migrates_legacy_skill_and_hook_state_and_removes_old_files() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let host = ExtensionHost::Codex;
    write(
        &home.join(".prelay").join("extensions.json"),
        &serde_json::to_string(&json!({
            "demo": {
                "version": "v1.0.0",
                "commitSha": "abc",
                "targets": {
                    "codexCli": {"skills": ["foo"], "mcps": [], "hooks": ["SessionStart"]}
                }
            }
        }))
        .unwrap(),
    );
    write(
        &home.join(".agents").join(".prelay").join("skill.json"),
        &serde_json::to_string(&json!({
            "demo": {"version": "v1.0.0", "commitSha": "abc", "skills": ["foo"]}
        }))
        .unwrap(),
    );
    write(
        &home.join(".codex").join("hooks.json"),
        &serde_json::to_string(&json!({
            "hooks": {
                "SessionStart": [{
                    "hooks": [{"type": "command", "command": "demo-hook"}]
                }]
            }
        }))
        .unwrap(),
    );
    let hash = crate::extensions::hooks::handlers(home).unwrap()[0]
        .hash
        .clone();
    write(
        &home.join(".codex").join(".prelay").join("hooks.json"),
        &serde_json::to_string(&json!({
            "demo": {"version": "v1.0.0", "commitSha": "abc", "hashes": [hash]}
        }))
        .unwrap(),
    );

    let _lock = lock(home, host).unwrap();
    ensure_migrated_locked(home).unwrap();
    let state = HostState::load_locked(home, host).unwrap();

    let plugin = state.plugin("demo").unwrap();
    assert_eq!(plugin.version, "v1.0.0");
    assert!(plugin.artifacts.iter().any(|artifact| matches!(
        artifact,
        PluginArtifact::Skill { names, .. } if names == &vec!["foo".to_string()]
    )));
    assert!(plugin.artifacts.iter().any(|artifact| matches!(
        artifact,
        PluginArtifact::Hook { handler_hash, .. } if handler_hash == &hash
    )));
    assert!(!home.join(".prelay").join("extensions.json").exists());
    assert!(!home
        .join(".agents")
        .join(".prelay")
        .join("skill.json")
        .exists());
    assert!(!home
        .join(".codex")
        .join(".prelay")
        .join("hooks.json")
        .exists());
    assert!(home.join(".prelay").join("backup").is_dir());
}

#[test]
fn migrates_legacy_rule_state_and_removes_old_file() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let host = ExtensionHost::Codex;
    write(&home.join(".codex").join("AGENTS.md"), "# Rules\n");
    write(
        &home.join(".codex").join(".prelay").join("rule.json"),
        &serde_json::to_string(&json!({
            "package": "development-rules",
            "version": "v1.0.0",
            "commitSha": "abc"
        }))
        .unwrap(),
    );

    let _lock = lock(home, host).unwrap();
    ensure_migrated_locked(home).unwrap();
    let state = HostState::load_locked(home, host).unwrap();

    let entry = state
        .rule_entry(&home.join(".codex").join("AGENTS.md"))
        .unwrap();
    assert_eq!(entry.package, "development-rules");
    assert_eq!(
        entry.content_hash,
        crate::extensions::host_state::hash_bytes(b"# Rules\n")
    );
    assert!(!home
        .join(".codex")
        .join(".prelay")
        .join("rule.json")
        .exists());
}

#[test]
fn migrates_legacy_mcp_state_with_a_matching_fingerprint() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let host = ExtensionHost::Codex;
    write(
        &home.join(".codex").join("config.toml"),
        "[mcp_servers.demo]\ncommand = \"demo\"\nenabled = true\n",
    );
    write(
        &home.join(".codex").join(".prelay").join("mcp.json"),
        &serde_json::to_string(&json!({
            "demo": {
                "serverName": "demo",
                "version": "v1.0.0",
                "commitSha": "abc",
                "manifest": {
                    "name": "demo",
                    "transport": {
                        "type": "stdio",
                        "command": ["demo"],
                        "cwd": null,
                        "environment": {},
                        "enabled": true,
                        "timeoutMs": null
                    }
                }
            }
        }))
        .unwrap(),
    );

    let _lock = lock(home, host).unwrap();
    ensure_migrated_locked(home).unwrap();
    let state = HostState::load_locked(home, host).unwrap();
    let plugin = state.plugin("demo").unwrap();
    let artifact = plugin
        .artifacts
        .iter()
        .find_map(|artifact| match artifact {
            PluginArtifact::Mcp {
                server_name,
                manifest_hash,
                ..
            } => Some((server_name, manifest_hash)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        crate::extensions::mcp::health(home, artifact.0, artifact.1).unwrap(),
        super::ArtifactHealth::Managed
    );
}

#[test]
fn marks_migration_partial_when_artifact_versions_disagree() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let host = ExtensionHost::Codex;
    write(
        &home.join(".prelay").join("extensions.json"),
        &serde_json::to_string(&json!({
            "demo": {
                "version": "v1.0.0",
                "commitSha": "abc",
                "targets": {"codexCli": {"skills": ["foo"]}}
            }
        }))
        .unwrap(),
    );
    write(
        &home.join(".agents").join(".prelay").join("skill.json"),
        &serde_json::to_string(&json!({
            "demo": {"version": "v2.0.0", "commitSha": "def", "skills": ["foo"]}
        }))
        .unwrap(),
    );

    let _lock = lock(home, host).unwrap();
    ensure_migrated_locked(home).unwrap();
    let state = HostState::load_locked(home, host).unwrap();

    assert_eq!(state.plugin("demo").unwrap().state, PluginState::Installing);
}
