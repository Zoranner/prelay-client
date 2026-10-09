use std::fs;

use tempfile::tempdir;

use super::super::{
    scan_user_items_with_installation, uninstall_user_item_with_installation, AgentClient,
    AgentItemKind, AgentItemSource,
};
use super::write;

#[test]
fn marks_extension_installed_mcp_as_team() {
    let directory = tempdir().unwrap();
    write(
        directory.path().join(".codex").join("config.toml"),
        r#"
[mcp_servers.filesystem]
command = "uvx"

[mcp_servers.manual]
command = "manual-mcp"
"#,
    );
    write(
        directory
            .path()
            .join(".codex")
            .join(".prelay")
            .join("mcp.json"),
        r#"{
  "filesystem-mcp": {
    "serverName": "filesystem",
    "version": "v1.0.0",
    "commitSha": "abc123"
  }
}"#,
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    let items = &snapshot.clients[0].items;
    let mcp = |name: &str| {
        items
            .iter()
            .find(|item| item.kind == AgentItemKind::Mcp && item.name == name)
            .unwrap()
    };

    let installed = mcp("filesystem");
    assert_eq!(installed.source, AgentItemSource::Team);
    assert_eq!(installed.version.as_deref(), Some("v1.0.0"));
    assert_eq!(installed.package.as_deref(), Some("filesystem-mcp"));

    let manual = mcp("manual");
    assert_eq!(manual.source, AgentItemSource::Personal);
    assert_eq!(manual.version, None);
    assert_eq!(manual.package, None);
}

#[test]
fn uninstalling_a_managed_mcp_only_removes_its_configuration() {
    let directory = tempdir().unwrap();
    write(
        directory.path().join(".codex").join("config.toml"),
        r#"
[mcp_servers.filesystem]
command = "uvx"

[mcp_servers.keep]
command = "keep"
"#,
    );
    write(
        directory
            .path()
            .join(".codex")
            .join(".prelay")
            .join("mcp.json"),
        r#"{
  "filesystem-mcp": {
    "serverName": "filesystem",
    "version": "v1.0.0",
    "commitSha": "abc123"
  }
}"#,
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    let item = snapshot.clients[0]
        .items
        .iter()
        .find(|item| item.kind == AgentItemKind::Mcp && item.name == "filesystem")
        .unwrap()
        .clone();
    assert_eq!(item.source, AgentItemSource::Team);

    uninstall_user_item_with_installation(
        directory.path(),
        AgentClient::CodexCli,
        item.kind,
        &item.name,
        &item.source_path,
        |client| client == AgentClient::CodexCli,
    )
    .unwrap();

    let config = fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap();
    assert!(!config.contains("filesystem"));
    assert!(config.contains("keep"));
}
