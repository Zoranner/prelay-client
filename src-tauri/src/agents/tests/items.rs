use std::fs;

use tempfile::tempdir;

use super::super::{
    scan_user_items_with_installation, uninstall_user_item_with_installation, AgentClient,
    AgentItemKind, AgentItemSource, AgentItemStatus,
};
use super::write;

#[test]
fn scans_user_level_codex_items_and_distinguishes_disabled_entries() {
    let directory = tempdir().unwrap();
    write(
        directory.path().join(".codex").join("config.toml"),
        r#"
[mcp_servers.research]
command = "prelay-search"

[mcp_servers.retired]
enabled = false
command = "retired-search"

"#,
    );
    write(
        directory
            .path()
            .join(".agents")
            .join("skills")
            .join("web-research")
            .join("SKILL.md"),
        "---\nname: web-research\n---\n",
    );
    write(
        directory
            .path()
            .join(".agents")
            .join(".prelay")
            .join("skills")
            .join("7765622d7265736561726368.json"),
        r#"{
  "package": "web-research",
  "version": "v1.2.0",
  "commitSha": "abc123",
  "roots": ["web-research"]
}"#,
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    assert_eq!(snapshot.clients.len(), 1);
    let codex = snapshot
        .clients
        .iter()
        .find(|client| client.client == AgentClient::CodexCli)
        .unwrap();

    assert!(codex.items.iter().any(|item| {
        item.kind == AgentItemKind::Mcp
            && item.name == "research"
            && item.status == AgentItemStatus::Enabled
    }));
    assert!(codex.items.iter().any(|item| {
        item.kind == AgentItemKind::Mcp
            && item.name == "retired"
            && item.status == AgentItemStatus::Disabled
    }));
    assert!(codex.items.iter().any(|item| {
        item.kind == AgentItemKind::Skill
            && item.name == "web-research"
            && item.status == AgentItemStatus::Enabled
            && item.source == AgentItemSource::Team
            && item.version.as_deref() == Some("v1.2.0")
    }));
}

#[test]
fn marks_unmanaged_local_skill_as_personal_without_version() {
    let directory = tempdir().unwrap();
    write(
        directory.path().join(".codex").join("config.toml"),
        "[mcp_servers.research]\ncommand = \"prelay-search\"\n",
    );
    write(
        directory
            .path()
            .join(".agents")
            .join("skills")
            .join("manual")
            .join("SKILL.md"),
        "# Manual\n",
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    let skill = snapshot.clients[0]
        .items
        .iter()
        .find(|item| item.kind == AgentItemKind::Skill)
        .unwrap();

    assert_eq!(skill.source, AgentItemSource::Personal);
    assert_eq!(skill.version, None);
}

#[test]
fn records_invalid_client_configuration_as_one_error() {
    let directory = tempdir().unwrap();
    write(
        directory.path().join(".codex").join("config.toml"),
        "[mcp_servers.invalid",
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    let codex = snapshot
        .clients
        .iter()
        .find(|client| client.client == AgentClient::CodexCli)
        .unwrap();

    assert_eq!(codex.items.len(), 1);
    assert_eq!(codex.items[0].status, AgentItemStatus::Error);
}

#[test]
fn ignores_host_managed_codex_plugin_configuration() {
    let directory = tempdir().unwrap();
    write(
        directory.path().join(".codex").join("config.toml"),
        "[plugins.\"browser@openai-bundled\"]\nenabled = true\n",
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });

    assert!(snapshot.clients[0].items.is_empty());
}

#[test]
fn uses_native_separators_for_skill_source_paths() {
    let directory = tempdir().unwrap();
    write(
        directory.path().join(".codex").join("config.toml"),
        "[mcp_servers.research]\ncommand = \"prelay-search\"\n",
    );
    write(
        directory
            .path()
            .join(".agents")
            .join("skills")
            .join("cavecrew")
            .join("SKILL.md"),
        "---\nname: cavecrew\n---\n",
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    let skill = snapshot
        .clients
        .iter()
        .find(|client| client.client == AgentClient::CodexCli)
        .and_then(|client| {
            client
                .items
                .iter()
                .find(|item| item.kind == AgentItemKind::Skill)
        })
        .unwrap();

    assert_eq!(
        skill.source_path,
        directory
            .path()
            .join(".agents")
            .join("skills")
            .join("cavecrew")
            .display()
            .to_string()
    );
}

#[test]
fn uninstalls_codex_items_without_removing_other_entries() {
    let directory = tempdir().unwrap();
    let config_path = directory.path().join(".codex").join("config.toml");
    write(
        &config_path,
        r#"
[mcp_servers.keep]
command = "keep"

[mcp_servers.remove]
command = "remove"

"#,
    );
    write(
        directory
            .path()
            .join(".agents")
            .join("skills")
            .join("remove")
            .join("SKILL.md"),
        "---\nname: remove\n---\n",
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    let codex = snapshot
        .clients
        .iter()
        .find(|client| client.client == AgentClient::CodexCli)
        .unwrap();
    for kind in [AgentItemKind::Mcp, AgentItemKind::Skill] {
        let item = codex
            .items
            .iter()
            .find(|item| item.kind == kind && item.name == "remove")
            .unwrap();
        uninstall_user_item_with_installation(
            directory.path(),
            AgentClient::CodexCli,
            item.kind,
            &item.name,
            &item.source_path,
            |client| client == AgentClient::CodexCli,
        )
        .unwrap();
    }

    let items = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    })
    .clients
    .remove(0)
    .items;
    assert!(items.iter().any(|item| item.name == "keep"));
    assert!(items.iter().all(|item| item.name != "remove"));
}

#[test]
fn omits_clients_when_their_commands_are_unavailable() {
    let directory = tempdir().unwrap();

    assert!(
        scan_user_items_with_installation(directory.path(), |_| false)
            .clients
            .is_empty()
    );
}

#[test]
fn includes_installed_client_without_extensions() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join(".codex")).unwrap();

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    assert_eq!(snapshot.clients.len(), 1);
    assert_eq!(snapshot.clients[0].client, AgentClient::CodexCli);
    assert!(snapshot.clients[0].items.is_empty());
}

#[test]
fn includes_chatgpt_when_only_the_desktop_app_is_installed() {
    let directory = tempdir().unwrap();

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::ChatGpt
    });

    assert_eq!(snapshot.clients.len(), 1);
    assert_eq!(snapshot.clients[0].client, AgentClient::ChatGpt);
    assert!(snapshot.clients[0].items.is_empty());
}
