use std::{collections::BTreeSet, fs};

use prelay_protocol::{ExtensionMcpManifest, ExtensionMcpTransport};
use tempfile::tempdir;

use super::{install_mcp, mcp_installation_status, retain_listed_mcp_packages, McpInstallAction};
use crate::agents::AgentClient;

#[test]
fn removing_an_unpublished_package_clears_only_its_mcp_state() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };
    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap();

    retain_listed_mcp_packages(directory.path(), &[AgentClient::CodexCli], &BTreeSet::new())
        .unwrap();

    let state: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".codex")
                .join(".prelay")
                .join("mcp.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(state, serde_json::json!({}));
    let config: toml::Value = toml::from_str(
        &fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap(),
    )
    .unwrap();
    assert!(config["mcp_servers"]["filesystem"].is_table());
}

#[test]
fn replacing_a_managed_mcp_server_transfers_its_version_record() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };
    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "first-package",
        "v1.0.0",
        "first",
        &manifest,
        false,
    )
    .unwrap();
    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "second-package",
        "v1.0.0",
        "second",
        &manifest,
        true,
    )
    .unwrap();

    let first = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "first-package",
        "v1.0.0",
        "first",
    )
    .unwrap();
    assert_eq!(first.action, McpInstallAction::Install);
    let second = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "second-package",
        "v1.0.0",
        "second",
    )
    .unwrap();
    assert_eq!(second.action, McpInstallAction::Installed);
}

#[test]
fn updating_a_package_with_a_renamed_server_removes_its_old_configuration() {
    let directory = tempdir().unwrap();
    let old_manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };
    let new_manifest = ExtensionMcpManifest {
        name: "workspace-files".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };
    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "first",
        &old_manifest,
        false,
    )
    .unwrap();
    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.1.0",
        "second",
        &new_manifest,
        false,
    )
    .unwrap();

    let config: toml::Value = toml::from_str(
        &fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap(),
    )
    .unwrap();
    assert!(config["mcp_servers"].get("filesystem").is_none());
    assert!(config["mcp_servers"]["workspace-files"].is_table());
}

#[test]
fn reports_update_when_a_managed_mcp_configuration_has_changed() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };
    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap();
    fs::write(
        directory.path().join(".codex").join("config.toml"),
        r#"[mcp_servers.filesystem]
command = "other-command"
enabled = true
"#,
    )
    .unwrap();

    let status = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
    )
    .unwrap();
    assert_eq!(status.action, McpInstallAction::Update);
    assert_eq!(status.clients, vec![AgentClient::CodexCli]);
}

#[test]
fn reports_update_when_legacy_mcp_state_has_no_manifest_snapshot() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };
    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap();
    fs::write(
        directory
            .path()
            .join(".codex")
            .join(".prelay")
            .join("mcp.json"),
        r#"{
            "filesystem-mcp": {
                "serverName": "filesystem",
                "version": "v1.0.0",
                "commitSha": "commit"
            }
        }"#,
    )
    .unwrap();

    let status = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
    )
    .unwrap();
    assert_eq!(status.action, McpInstallAction::Update);
    assert_eq!(status.clients, vec![AgentClient::CodexCli]);
}

#[test]
fn rejects_a_multi_client_install_before_writing_any_target_with_a_conflict() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join(".claude.json"),
        r#"{
            "mcpServers": {
                "filesystem": { "type": "stdio", "command": "manual-command" }
            }
        }"#,
    )
    .unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };

    assert!(install_mcp(
        directory.path(),
        &[AgentClient::CodexCli, AgentClient::ClaudeCode],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .is_err());
    assert!(!directory.path().join(".codex").join("config.toml").exists());
}
