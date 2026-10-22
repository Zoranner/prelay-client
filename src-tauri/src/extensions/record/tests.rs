use std::{collections::BTreeMap, fs};

use tempfile::tempdir;

use crate::agents::{AgentClient, AgentItemKind};

use super::{
    managed_package_names, plugin_items, read_installed_extensions, upsert_extension_record,
    InstalledTargetRecord,
};

#[test]
fn upserts_a_package_record() {
    let directory = tempdir().unwrap();
    let targets = BTreeMap::from([
        (
            "codexCli".to_string(),
            InstalledTargetRecord {
                skills: vec!["using-superpowers".to_string()],
                hooks: vec!["SessionStart:startup".to_string()],
                ..Default::default()
            },
        ),
        (
            "openCode".to_string(),
            InstalledTargetRecord {
                skills: vec!["using-superpowers".to_string()],
                ..Default::default()
            },
        ),
    ]);

    upsert_extension_record(directory.path(), "superpowers", "v6.3.1", "commit", targets).unwrap();
    let installed = read_installed_extensions(directory.path()).unwrap();
    let record = installed.0.get("superpowers").unwrap();
    assert_eq!(record.version, "v6.3.1");
    assert_eq!(
        record.targets["codexCli"].hooks,
        vec!["SessionStart:startup"]
    );
    assert_eq!(record.targets["openCode"].skills, vec!["using-superpowers"]);
}

#[test]
fn aggregates_a_managed_package_into_one_plugin_item() {
    let directory = tempdir().unwrap();
    let targets = BTreeMap::from([
        (
            "codexCli".to_string(),
            InstalledTargetRecord {
                skills: vec!["using-superpowers".to_string()],
                hooks: vec!["SessionStart".to_string()],
                ..Default::default()
            },
        ),
        (
            "openCode".to_string(),
            InstalledTargetRecord {
                skills: vec!["using-superpowers".to_string()],
                ..Default::default()
            },
        ),
    ]);
    upsert_extension_record(directory.path(), "superpowers", "v6.3.1", "commit", targets).unwrap();

    let managed = managed_package_names(directory.path(), AgentClient::CodexCli).unwrap();
    assert!(managed.contains("superpowers"));
    let items = plugin_items(directory.path(), AgentClient::CodexCli);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, AgentItemKind::Plugin);
    assert_eq!(
        items[0].members,
        vec!["Skill · using-superpowers", "Hook · SessionStart"]
    );
    assert_eq!(
        plugin_items(directory.path(), AgentClient::OpenCode)[0].members,
        vec!["Skill · using-superpowers"]
    );
}

#[test]
fn merges_legacy_records_on_the_first_write() {
    let directory = tempdir().unwrap();
    let skill_state = directory.path().join(".agents").join(".prelay");
    fs::create_dir_all(&skill_state).unwrap();
    fs::write(
        skill_state.join("skill.json"),
        serde_json::to_vec(&serde_json::json!({
            "superpowers": {
                "version": "v6.3.1",
                "commitSha": "commit",
                "skills": ["using-superpowers"]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let mcp_state = directory.path().join(".codex").join(".prelay");
    fs::create_dir_all(&mcp_state).unwrap();
    fs::write(
        mcp_state.join("mcp.json"),
        serde_json::to_vec(&serde_json::json!({
            "superpowers": {
                "serverName": "superpowers",
                "version": "v6.3.1",
                "commitSha": "commit"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let record = read_installed_extensions(directory.path()).unwrap();
    assert!(record.0.is_empty());
    upsert_extension_record(
        directory.path(),
        "new-package",
        "v1",
        "new-commit",
        BTreeMap::new(),
    )
    .unwrap();
    let record = read_installed_extensions(directory.path()).unwrap();
    assert_eq!(record.0["superpowers"].version, "v6.3.1");
    assert_eq!(
        record.0["superpowers"].targets["codexCli"].skills,
        vec!["using-superpowers"]
    );
    assert_eq!(
        record.0["superpowers"].targets["codexCli"].mcps,
        vec!["superpowers"]
    );
    assert!(directory
        .path()
        .join(".prelay")
        .join("extensions.json")
        .exists());
}
