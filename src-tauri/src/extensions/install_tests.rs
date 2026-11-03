use std::fs;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::{
    ExtensionArtifact, ExtensionFile, ExtensionInstallBundle, ExtensionKind, ExtensionVersion,
};
use tempfile::tempdir;

use crate::agents::AgentClient;

use crate::extensions::record::read_installed_extensions;

use super::install_prepared_bundle;

fn bundle(hook_json: &str) -> ExtensionInstallBundle {
    bundle_with_hook_files(vec![
        ExtensionFile {
            path: "hooks/hooks.json".to_string(),
            content_base64: BASE64.encode(hook_json),
        },
        ExtensionFile {
            path: "hooks/scripts/session-start.sh".to_string(),
            content_base64: BASE64.encode("#!/bin/sh\n"),
        },
    ])
}

fn bundle_with_hook_files(hook_files: Vec<ExtensionFile>) -> ExtensionInstallBundle {
    ExtensionInstallBundle {
        name: "superpowers".to_string(),
        version: ExtensionVersion {
            tag: "v6.3.1".to_string(),
            commit_sha: "commit".to_string(),
            updated_at: "2026-10-22T00:00:00Z".to_string(),
        },
        artifacts: vec![
            ExtensionArtifact {
                kind: ExtensionKind::Skill,
                files: vec![ExtensionFile {
                    path: "skills/using-superpowers/SKILL.md".to_string(),
                    content_base64: BASE64.encode("# Using superpowers"),
                }],
            },
            ExtensionArtifact {
                kind: ExtensionKind::Hook,
                files: hook_files,
            },
        ],
        skipped: vec!["README.md".to_string()],
    }
}

fn bundle_with_all_artifacts() -> ExtensionInstallBundle {
    ExtensionInstallBundle {
        name: "superpowers".to_string(),
        version: ExtensionVersion {
            tag: "v6.3.1".to_string(),
            commit_sha: "commit".to_string(),
            updated_at: "2026-10-22T00:00:00Z".to_string(),
        },
        artifacts: vec![
            ExtensionArtifact {
                kind: ExtensionKind::Mcp,
                files: vec![ExtensionFile {
                    path: "server.json".to_string(),
                    content_base64: BASE64.encode(
                        r#"{
                          "name": "superpowers",
                          "transport": {
                            "type": "stdio",
                            "command": ["superpowers-mcp"],
                            "cwd": null,
                            "environment": {},
                            "enabled": true,
                            "timeoutMs": null
                          }
                        }"#,
                    ),
                }],
            },
            ExtensionArtifact {
                kind: ExtensionKind::Skill,
                files: vec![ExtensionFile {
                    path: "skills/using-superpowers/SKILL.md".to_string(),
                    content_base64: BASE64.encode("# Using superpowers"),
                }],
            },
            ExtensionArtifact {
                kind: ExtensionKind::Hook,
                files: vec![
                    ExtensionFile {
                        path: "hooks/hooks.json".to_string(),
                        content_base64: BASE64.encode(HOOK_JSON),
                    },
                    ExtensionFile {
                        path: "hooks/scripts/session-start.sh".to_string(),
                        content_base64: BASE64.encode("#!/bin/sh\n"),
                    },
                ],
            },
        ],
        skipped: vec!["README.md".to_string()],
    }
}

const HOOK_JSON: &str = r#"{
  "hooks": {
    "SessionStart": [{
      "hooks": [{"type": "command", "command": "prelay-session"}]
    }]
  }
}"#;

#[test]
fn installs_skill_and_hook_and_records_both_targets() {
    let directory = tempdir().unwrap();
    let review = install_prepared_bundle(
        directory.path(),
        &bundle(HOOK_JSON),
        &[AgentClient::CodexCli],
        false,
    )
    .unwrap();

    assert_eq!(review.len(), 1);
    assert!(directory
        .path()
        .join(".agents")
        .join("skills")
        .join("using-superpowers")
        .join("SKILL.md")
        .is_file());
    let hooks: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(".codex").join("hooks.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        hooks["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "prelay-session"
    );
    assert!(directory
        .path()
        .join(".prelay")
        .join("packages")
        .join("superpowers")
        .join("commit")
        .join("hooks")
        .join("scripts")
        .join("session-start.sh")
        .is_file());
    let record = read_installed_extensions(directory.path()).unwrap();
    let target = &record.0["superpowers"].targets["codexCli"];
    assert_eq!(target.skills, vec!["using-superpowers"]);
    assert_eq!(target.hooks, vec!["SessionStart"]);
}

#[test]
fn preflight_failure_does_not_write_the_skill_artifact() {
    let directory = tempdir().unwrap();
    let hooks_path = directory.path().join(".codex").join("hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    fs::write(&hooks_path, "{broken").unwrap();

    let result = install_prepared_bundle(
        directory.path(),
        &bundle(HOOK_JSON),
        &[AgentClient::CodexCli],
        false,
    );

    assert!(result.is_err());
    assert!(!directory
        .path()
        .join(".agents")
        .join("skills")
        .join("using-superpowers")
        .exists());
    assert!(!directory.path().join(".prelay").exists());
    assert_eq!(fs::read_to_string(hooks_path).unwrap(), "{broken");
}

#[test]
fn installs_skill_mcp_and_hook_together() {
    let directory = tempdir().unwrap();
    let review = install_prepared_bundle(
        directory.path(),
        &bundle_with_all_artifacts(),
        &[AgentClient::CodexCli],
        false,
    )
    .unwrap();

    assert_eq!(review.len(), 1);
    assert!(directory
        .path()
        .join(".agents")
        .join("skills")
        .join("using-superpowers")
        .join("SKILL.md")
        .is_file());
    assert!(directory
        .path()
        .join(".codex")
        .join("config.toml")
        .is_file());
    assert!(directory.path().join(".codex").join("hooks.json").is_file());
    assert!(directory
        .path()
        .join(".prelay")
        .join("packages")
        .join("superpowers")
        .join("commit")
        .join("hooks")
        .join("scripts")
        .join("session-start.sh")
        .is_file());

    let record = read_installed_extensions(directory.path()).unwrap();
    let target = &record.0["superpowers"].targets["codexCli"];
    assert_eq!(target.skills, vec!["using-superpowers"]);
    assert_eq!(target.mcps, vec!["superpowers"]);
    assert_eq!(target.hooks, vec!["SessionStart"]);
}

#[test]
fn chatgpt_installs_skill_mcp_and_hook() {
    let directory = tempdir().unwrap();
    let review = install_prepared_bundle(
        directory.path(),
        &bundle_with_all_artifacts(),
        &[AgentClient::ChatGpt],
        false,
    )
    .unwrap();

    assert_eq!(review.len(), 1);
    assert!(directory.path().join(".codex").join("hooks.json").is_file());
    assert!(directory
        .path()
        .join(".prelay")
        .join("packages")
        .join("superpowers")
        .join("commit")
        .join("hooks")
        .is_dir());
    assert!(directory
        .path()
        .join(".agents")
        .join("skills")
        .join("using-superpowers")
        .join("SKILL.md")
        .is_file());
    assert!(directory
        .path()
        .join(".codex")
        .join("config.toml")
        .is_file());

    let record = read_installed_extensions(directory.path()).unwrap();
    let target = &record.0["superpowers"].targets["chatgpt"];
    assert_eq!(target.skills, vec!["using-superpowers"]);
    assert_eq!(target.mcps, vec!["superpowers"]);
    assert_eq!(target.hooks, vec!["SessionStart"]);
}

#[test]
fn shares_codex_hook_configuration_between_codex_cli_and_chatgpt() {
    let directory = tempdir().unwrap();
    let review = install_prepared_bundle(
        directory.path(),
        &bundle_with_all_artifacts(),
        &[AgentClient::CodexCli, AgentClient::ChatGpt],
        false,
    )
    .unwrap();

    assert_eq!(review.len(), 1);
    let hooks: serde_json::Value = serde_json::from_slice(
        &std::fs::read(directory.path().join(".codex").join("hooks.json")).unwrap(),
    )
    .unwrap();
    let handlers = hooks["hooks"]["SessionStart"][0]["hooks"]
        .as_array()
        .unwrap();
    assert_eq!(handlers.len(), 1);
    assert_eq!(handlers[0]["command"], "prelay-session");

    let record = read_installed_extensions(directory.path()).unwrap();
    for client in ["codexCli", "chatgpt"] {
        assert_eq!(
            record.0["superpowers"].targets[client].hooks,
            vec!["SessionStart"]
        );
    }
}

#[test]
fn rejects_a_hook_bundle_without_the_definition_file() {
    let directory = tempdir().unwrap();
    let result = install_prepared_bundle(
        directory.path(),
        &bundle_with_hook_files(vec![ExtensionFile {
            path: "hooks/scripts/session-start.sh".to_string(),
            content_base64: BASE64.encode("#!/bin/sh\n"),
        }]),
        &[AgentClient::CodexCli],
        false,
    );
    assert!(result.is_err());
    assert!(!directory.path().join(".codex").join("hooks.json").exists());
}

#[test]
fn rejects_a_hook_bundle_with_a_traversal_path() {
    let directory = tempdir().unwrap();
    let result = install_prepared_bundle(
        directory.path(),
        &bundle_with_hook_files(vec![
            ExtensionFile {
                path: "hooks/hooks.json".to_string(),
                content_base64: BASE64.encode(HOOK_JSON),
            },
            ExtensionFile {
                path: "hooks/../escape.sh".to_string(),
                content_base64: BASE64.encode("#!/bin/sh\n"),
            },
        ]),
        &[AgentClient::CodexCli],
        false,
    );
    assert!(result.is_err());
}
