use std::fs;
#[cfg(windows)]
use std::time::{Duration, Instant};

use tempfile::tempdir;

#[cfg(windows)]
use super::super::command_client_version;
use super::super::{
    agent_client_statuses_with, command_path_in, command_version_from_output,
    scan_user_items_with_installation, AgentClient, AgentClientVersion, AgentItemKind,
    AgentItemStatus,
};
#[cfg(windows)]
use super::super::{chatgpt_offline_install_version, chatgpt_package_version, newest_version};
use super::write;

#[test]
fn includes_every_registered_agent_client_when_installed() {
    let directory = tempdir().unwrap();

    let snapshot = scan_user_items_with_installation(directory.path(), |_| true);
    let clients = snapshot
        .clients
        .iter()
        .map(|client| serde_json::to_value(client.client).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        clients,
        vec![
            serde_json::json!("codexCli"),
            serde_json::json!("chatgpt"),
            serde_json::json!("claudeCode"),
            serde_json::json!("openCode"),
        ]
    );
}

#[test]
fn reports_every_registered_client_with_its_installation_status() {
    let statuses = agent_client_statuses_with(
        |client| client == AgentClient::OpenCode,
        |clients| {
            assert_eq!(clients, vec![AgentClient::OpenCode]);
            vec![AgentClientVersion {
                client: AgentClient::OpenCode,
                version: Some("1.2.3".to_string()),
            }]
        },
    );

    assert_eq!(statuses.len(), 4);
    assert!(!statuses[0].installed);
    assert!(!statuses[1].installed);
    assert_eq!(statuses[2].client, AgentClient::ClaudeCode);
    assert!(!statuses[2].installed);
    assert_eq!(statuses[3].client, AgentClient::OpenCode);
    assert!(statuses[3].installed);
    assert_eq!(statuses[3].version.as_deref(), Some("1.2.3"));
}

#[test]
fn scans_only_opencode_jsonc() {
    let directory = tempdir().unwrap();
    write(
        directory
            .path()
            .join(".config")
            .join("opencode")
            .join("config.json"),
        r#"{ "mcp": { "ignore-config": {} } }"#,
    );
    write(
        directory
            .path()
            .join(".config")
            .join("opencode")
            .join("opencode.json"),
        r#"{ "plugin": ["ignore-opencode"] }"#,
    );
    write(
        directory
            .path()
            .join(".config")
            .join("opencode")
            .join("opencode.jsonc"),
        r#"// Global OpenCode configuration
{
  "mcp": {
    "prelay-search": { "type": "local", "command": ["prelay-search"] },
    "retired-search": { "type": "local", "enabled": false, "command": ["retired-search"] }
  },
}"#,
    );
    write(
        directory
            .path()
            .join(".config")
            .join("opencode")
            .join("skills")
            .join("legacy-skill")
            .join("SKILL.md"),
        "---\nname: legacy-skill\n---\n",
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

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::OpenCode
    });
    let opencode = snapshot.clients.first().unwrap();

    assert!(opencode.items.iter().any(|item| {
        item.kind == AgentItemKind::Mcp
            && item.name == "prelay-search"
            && item.status == AgentItemStatus::Enabled
    }));
    assert!(opencode.items.iter().any(|item| {
        item.kind == AgentItemKind::Mcp
            && item.name == "retired-search"
            && item.status == AgentItemStatus::Disabled
    }));
    assert!(opencode
        .items
        .iter()
        .all(|item| item.name != "ignore-config"));
    assert!(opencode
        .items
        .iter()
        .all(|item| item.name != "ignore-opencode"));
    assert!(opencode.items.iter().any(|item| {
        item.kind == AgentItemKind::Skill
            && item.name == "web-research"
            && item.status == AgentItemStatus::Enabled
    }));
    assert!(opencode
        .items
        .iter()
        .all(|item| item.name != "legacy-skill"));
}

#[test]
fn keeps_chatgpt_desktop_and_codex_cli_as_separate_clients() {
    let directory = tempdir().unwrap();

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        matches!(client, AgentClient::ChatGpt | AgentClient::CodexCli)
    });

    assert_eq!(snapshot.clients.len(), 2);
    assert!(snapshot
        .clients
        .iter()
        .any(|client| client.client == AgentClient::ChatGpt));
    assert!(snapshot
        .clients
        .iter()
        .any(|client| client.client == AgentClient::CodexCli));
}

#[test]
fn initial_scan_does_not_probe_client_versions() {
    let directory = tempdir().unwrap();

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });

    assert_eq!(snapshot.clients.len(), 1);
    assert_eq!(snapshot.clients[0].client, AgentClient::CodexCli);
    assert_eq!(snapshot.clients[0].version, None);
    assert!(snapshot.clients[0].items.is_empty());
}

#[test]
fn extracts_a_semantic_version_from_command_output() {
    assert_eq!(
        command_version_from_output("codex-cli 0.83.0\n"),
        Some("0.83.0".to_string())
    );
    assert_eq!(
        command_version_from_output("Agent CLI v2.1.17\n"),
        Some("2.1.17".to_string())
    );
    assert_eq!(command_version_from_output("unknown"), None);
}

#[cfg(windows)]
#[test]
fn reads_a_version_from_a_command_that_finishes_within_five_seconds() {
    let directory = tempdir().unwrap();
    let command = directory.path().join("slow-version.cmd");
    write(
        &command,
        "@powershell.exe -NoProfile -Command \"Start-Sleep -Milliseconds 2800\"\r\n@echo agent 1.2.3\r\n",
    );

    let started_at = Instant::now();
    assert_eq!(command_client_version(&command), Some("1.2.3".to_string()));
    assert!(started_at.elapsed() >= Duration::from_millis(2_800));
}

#[cfg(windows)]
#[test]
fn reads_the_version_from_a_chatgpt_package_full_name() {
    assert_eq!(
        chatgpt_package_version("OpenAI.Codex_26.818.8289.0_x64__2p2nqsd0c76g0").as_deref(),
        Some("26.818.8289.0")
    );
    assert_eq!(
        chatgpt_package_version("NotCodex_99.0.0.0_x64__2p2nqsd0c76g0"),
        None
    );
    assert_eq!(
        chatgpt_package_version("OpenAI.Codex_26.818_x64__2p2nqsd0c76g0"),
        None
    );
}

#[cfg(windows)]
#[test]
fn matches_the_offline_chatgpt_installation_registration() {
    assert_eq!(
        chatgpt_offline_install_version("ChatGPT 26.901.6511.0", "OpenAI", "26.901.6511.0")
            .as_deref(),
        Some("26.901.6511.0")
    );
    assert_eq!(
        chatgpt_offline_install_version("ChatGPT 26.901.6511.0", "openai", "26.901.6511.0")
            .as_deref(),
        Some("26.901.6511.0")
    );
    assert_eq!(
        chatgpt_offline_install_version("ChatGPT 26.901.6511.0", "Other", "26.901.6511.0"),
        None
    );
    assert_eq!(
        chatgpt_offline_install_version("Codex 0.9.0", "OpenAI", "0.9.0"),
        None
    );
}

#[cfg(windows)]
#[test]
fn uses_the_newest_version_across_chatgpt_installation_sources() {
    assert_eq!(
        newest_version(["26.803.10989.0".to_string(), "26.901.6511.0".to_string(),]).as_deref(),
        Some("26.901.6511.0")
    );
    assert_eq!(newest_version(Vec::<String>::new()), None);
}

#[test]
fn omits_configuration_when_the_agent_command_is_unavailable() {
    let directory = tempdir().unwrap();
    let snapshot = scan_user_items_with_installation(directory.path(), |_| false);

    assert!(snapshot.clients.is_empty());
}

#[test]
fn finds_windows_command_shims_on_path() {
    let directory = tempdir().unwrap();
    let bin = directory.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("codex.cmd"), "").unwrap();

    assert!(command_path_in("codex", &[bin], &[".exe".to_string(), ".cmd".to_string()]).is_some());
    assert!(command_path_in("codex", &[], &[".exe".to_string(), ".cmd".to_string()]).is_none());
}
