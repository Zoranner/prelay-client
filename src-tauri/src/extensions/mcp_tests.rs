use std::{fs, path::MAIN_SEPARATOR};

use prelay_protocol::{ExtensionMcpManifest, ExtensionMcpTransport};
use tempfile::tempdir;

use super::{install_mcp, mcp_installation_status};
use crate::agents::AgentClient;
use crate::extensions::ExtensionInstallAction;

#[test]
fn writes_codex_mcp_configuration_and_tracks_the_package_version() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: [("GITHUB_TOKEN".to_string(), "GITHUB_TOKEN".to_string())].into(),
            enabled: true,
            timeout_ms: Some(30_000),
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

    let config: toml::Value = toml::from_str(
        &fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap(),
    )
    .unwrap();
    let server = &config["mcp_servers"]["filesystem"];
    assert_eq!(server["command"].as_str(), Some("uvx"));
    assert_eq!(
        server["args"].as_array().unwrap(),
        &[toml::Value::String("mcp-server-filesystem".to_string())]
    );
    assert_eq!(
        server["env_vars"].as_array().unwrap(),
        &[toml::Value::String("GITHUB_TOKEN".to_string())]
    );
    assert_eq!(server["tool_timeout_sec"].as_integer(), Some(30));

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
    assert_eq!(state["filesystem-mcp"]["serverName"], "filesystem");
    assert_eq!(state["filesystem-mcp"]["version"], "v1.0.0");

    let status = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
    )
    .unwrap();
    assert_eq!(status.action, ExtensionInstallAction::Installed);
    assert_eq!(status.clients, vec![AgentClient::CodexCli]);
}

#[test]
fn writes_opencode_stdio_mcp_configuration_without_replacing_other_state() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join(".config").join("opencode")).unwrap();
    fs::write(
        directory
            .path()
            .join(".config")
            .join("opencode")
            .join("opencode.jsonc"),
        r#"{ "theme": "system", "mcp": { "keep": { "type": "local", "command": ["keep"] } } }"#,
    )
    .unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["uvx".to_string(), "mcp-server-filesystem".to_string()],
            cwd: None,
            environment: [("GITHUB_TOKEN".to_string(), "GITHUB_TOKEN".to_string())].into(),
            enabled: true,
            timeout_ms: Some(30_000),
        },
    };

    install_mcp(
        directory.path(),
        &[AgentClient::OpenCode],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap();

    let config: serde_json::Value = json5::from_str(
        &fs::read_to_string(
            directory
                .path()
                .join(".config")
                .join("opencode")
                .join("opencode.jsonc"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(config["theme"], "system");
    assert!(config["mcp"]["keep"].is_object());
    assert_eq!(config["mcp"]["filesystem"]["type"], "local");
    assert_eq!(
        config["mcp"]["filesystem"]["command"],
        serde_json::json!(["uvx", "mcp-server-filesystem"])
    );
    assert_eq!(
        config["mcp"]["filesystem"]["environment"]["GITHUB_TOKEN"],
        "{env:GITHUB_TOKEN}"
    );
    assert_eq!(config["mcp"]["filesystem"]["enabled"], true);
    assert_eq!(config["mcp"]["filesystem"]["timeout"], 30_000);
    assert!(directory
        .path()
        .join(".config")
        .join("opencode")
        .join(".prelay")
        .join("mcp.json")
        .is_file());
}

#[test]
fn expands_user_directory_variables_in_command_arguments() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "filesystem".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec![
                "uvx".to_string(),
                "mcp-server-filesystem".to_string(),
                "%USERPROFILE%\\Documents".to_string(),
                "%APPDATA%\\SomeTool".to_string(),
                "%%USERPROFILE%%\\Documents".to_string(),
            ],
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

    let config: toml::Value = toml::from_str(
        &fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap(),
    )
    .unwrap();
    let args = config["mcp_servers"]["filesystem"]["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    let home = directory.path().display().to_string();
    let documents = format!("{home}{MAIN_SEPARATOR}Documents");
    let tool =
        format!("{home}{MAIN_SEPARATOR}AppData{MAIN_SEPARATOR}Roaming{MAIN_SEPARATOR}SomeTool");
    assert_eq!(args[0], "mcp-server-filesystem");
    assert_eq!(args[1], documents.as_str());
    assert_eq!(args[2], tool.as_str());
    assert_eq!(args[3], "%USERPROFILE%\\Documents");

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
    assert_eq!(
        state["filesystem-mcp"]["manifest"]["transport"]["command"][2],
        "%USERPROFILE%\\Documents"
    );

    let status = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
    )
    .unwrap();
    assert_eq!(status.action, ExtensionInstallAction::Installed);
}

#[test]
fn expands_prelay_home_in_the_command_binary() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "prelay_image".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec![
                "%PRELAYHOME%\\tools\\imagegen.exe".to_string(),
                "--verbose".to_string(),
            ],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };

    install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "prelay-image",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap();

    let config: toml::Value = toml::from_str(
        &fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap(),
    )
    .unwrap();
    let app_directory = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let expected = format!(
        "{}{MAIN_SEPARATOR}tools{MAIN_SEPARATOR}imagegen.exe",
        app_directory.display()
    );
    assert_eq!(
        config["mcp_servers"]["prelay_image"]["command"].as_str(),
        Some(expected.as_str())
    );
    assert_eq!(
        config["mcp_servers"]["prelay_image"]["args"]
            .as_array()
            .unwrap(),
        &[toml::Value::String("--verbose".to_string())]
    );

    let status = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "prelay-image",
        "v1.0.0",
        "commit",
    )
    .unwrap();
    assert_eq!(status.action, ExtensionInstallAction::Installed);
}

#[test]
fn rejects_unknown_directory_variables_before_writing_configuration() {
    let directory = tempdir().unwrap();
    let manifest = ExtensionMcpManifest {
        name: "prelay_image".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec!["imagegen".to_string(), "%PRELAY_HOME%\\tools".to_string()],
            cwd: None,
            environment: Default::default(),
            enabled: true,
            timeout_ms: None,
        },
    };

    let error = install_mcp(
        directory.path(),
        &[AgentClient::CodexCli],
        "prelay-image",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap_err();
    assert!(
        error.message.contains("%PRELAY_HOME%"),
        "unexpected error: {}",
        error.message
    );
    assert!(!directory.path().join(".codex").join("config.toml").exists());
}

#[test]
fn shares_codex_mcp_configuration_between_codex_cli_and_chatgpt() {
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
        &[AgentClient::CodexCli, AgentClient::ChatGpt],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap();

    let config: toml::Value = toml::from_str(
        &fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        config["mcp_servers"]["filesystem"]["command"].as_str(),
        Some("uvx")
    );
    assert!(directory
        .path()
        .join(".codex")
        .join(".prelay")
        .join("mcp.json")
        .is_file());
    assert!(!directory
        .path()
        .join(".config")
        .join("opencode")
        .join("opencode.jsonc")
        .exists());

    let status = mcp_installation_status(
        directory.path(),
        &[AgentClient::CodexCli, AgentClient::ChatGpt],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
    )
    .unwrap();
    assert_eq!(status.action, ExtensionInstallAction::Installed);
    assert_eq!(
        status.clients,
        vec![AgentClient::CodexCli, AgentClient::ChatGpt]
    );
}

#[test]
fn installs_chatgpt_target_into_codex_configuration() {
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
        &[AgentClient::ChatGpt],
        "filesystem-mcp",
        "v1.0.0",
        "commit",
        &manifest,
        false,
    )
    .unwrap();

    let config: toml::Value = toml::from_str(
        &fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        config["mcp_servers"]["filesystem"]["command"].as_str(),
        Some("uvx")
    );
}
