use std::{collections::BTreeMap, fs};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::ExtensionFile;
use tempfile::tempdir;

use crate::{
    agents::{agent_skill_target_roots, AgentClient},
    extensions::{
        hooks::install_hooks,
        record::{read_installed_extensions, upsert_extension_record, InstalledTargetRecord},
        skills::install_skill_files,
    },
};

use super::uninstall_plugin_for_client;

fn skill_file() -> ExtensionFile {
    ExtensionFile {
        path: "skills/using-superpowers/SKILL.md".to_string(),
        content_base64: BASE64.encode("# Using superpowers"),
    }
}

fn hook_file() -> ExtensionFile {
    ExtensionFile {
        path: "hooks/hooks.json".to_string(),
        content_base64: BASE64.encode(
            r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"prelay-session"}]}]}}"#,
        ),
    }
}

fn record_targets(skills: bool, hooks: bool) -> BTreeMap<String, InstalledTargetRecord> {
    BTreeMap::from([(
        "codexCli".to_string(),
        InstalledTargetRecord {
            skills: skills
                .then(|| "using-superpowers".to_string())
                .into_iter()
                .collect(),
            hooks: hooks
                .then(|| "SessionStart".to_string())
                .into_iter()
                .collect(),
            ..Default::default()
        },
    )])
}

#[test]
fn uninstall_plugin_removes_its_skill_hook_and_record() {
    let directory = tempdir().unwrap();
    let root = agent_skill_target_roots(&[AgentClient::CodexCli], directory.path())
        .into_iter()
        .next()
        .unwrap();
    install_skill_files(&root, "superpowers", "v1", "commit", &[skill_file()], false).unwrap();
    install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[hook_file()],
        false,
    )
    .unwrap();
    upsert_extension_record(
        directory.path(),
        "superpowers",
        "v1",
        "commit",
        record_targets(true, true),
    )
    .unwrap();

    uninstall_plugin_for_client(directory.path(), AgentClient::CodexCli, "superpowers").unwrap();

    assert!(!root.join("using-superpowers").exists());
    let hooks: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(".codex").join("hooks.json")).unwrap(),
    )
    .unwrap();
    let handlers = hooks["hooks"]["SessionStart"][0]["hooks"]
        .as_array()
        .unwrap();
    assert!(handlers.is_empty());
    assert!(read_installed_extensions(directory.path())
        .unwrap()
        .0
        .is_empty());
}

#[test]
fn uninstalling_one_client_keeps_a_shared_skill_for_the_other() {
    let directory = tempdir().unwrap();
    let root = agent_skill_target_roots(&[AgentClient::ChatGpt], directory.path())
        .into_iter()
        .next()
        .unwrap();
    install_skill_files(&root, "superpowers", "v1", "commit", &[skill_file()], false).unwrap();
    let shared = BTreeMap::from([
        (
            "codexCli".to_string(),
            InstalledTargetRecord {
                skills: vec!["using-superpowers".to_string()],
                ..Default::default()
            },
        ),
        (
            "chatgpt".to_string(),
            InstalledTargetRecord {
                skills: vec!["using-superpowers".to_string()],
                ..Default::default()
            },
        ),
    ]);
    upsert_extension_record(directory.path(), "superpowers", "v1", "commit", shared).unwrap();

    uninstall_plugin_for_client(directory.path(), AgentClient::ChatGpt, "superpowers").unwrap();

    assert!(root.join("using-superpowers").is_dir());
    let record = read_installed_extensions(directory.path()).unwrap();
    assert!(record.0["superpowers"].targets.contains_key("codexCli"));
    assert!(!record.0["superpowers"].targets.contains_key("chatgpt"));
}

#[test]
fn uninstalling_a_migrated_legacy_package_clears_shared_members() {
    let directory = tempdir().unwrap();
    let skill_root = directory.path().join(".agents").join("skills");
    fs::create_dir_all(skill_root.join("using-superpowers")).unwrap();
    fs::write(
        skill_root.join("using-superpowers").join("SKILL.md"),
        "# Using superpowers",
    )
    .unwrap();
    let state = directory.path().join(".agents").join(".prelay");
    fs::create_dir_all(&state).unwrap();
    fs::write(
        state.join("skill.json"),
        serde_json::to_vec(&serde_json::json!({
            "superpowers": {
                "version": "v1",
                "commitSha": "commit",
                "skills": ["using-superpowers"]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let codex_state = directory.path().join(".codex").join(".prelay");
    fs::create_dir_all(&codex_state).unwrap();
    fs::write(
        codex_state.join("mcp.json"),
        serde_json::to_vec(&serde_json::json!({
            "superpowers": {
                "serverName": "filesystem",
                "version": "v1",
                "commitSha": "commit"
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.path().join(".codex").join("config.toml"),
        "[mcp_servers.filesystem]\ncommand = \"uvx\"\n\n[mcp_servers.keep]\ncommand = \"keep\"\n",
    )
    .unwrap();
    upsert_extension_record(directory.path(), "other", "v1", "commit", BTreeMap::new()).unwrap();

    uninstall_plugin_for_client(directory.path(), AgentClient::CodexCli, "superpowers").unwrap();

    assert!(!skill_root.join("using-superpowers").exists());
    let config = fs::read_to_string(directory.path().join(".codex").join("config.toml")).unwrap();
    assert!(!config.contains("filesystem"));
    assert!(config.contains("keep"));
    let record = read_installed_extensions(directory.path()).unwrap();
    assert!(!record.0["superpowers"].targets.contains_key("codexCli"));
    assert!(record.0["superpowers"].targets.contains_key("chatgpt"));
}
