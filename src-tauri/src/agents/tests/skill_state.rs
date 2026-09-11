use std::fs;

use tempfile::tempdir;

use super::{
    super::{scan_user_items_with_installation, AgentClient, AgentItemKind, AgentItemSource},
    write,
};

#[test]
fn reads_team_skill_version_from_the_single_skill_root_state_file() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join(".codex")).unwrap();
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
            .join("skill.json"),
        r#"{
  "web-research": {
    "version": "v1.2.0",
    "commitSha": "abc123",
    "skills": ["web-research"]
  }
}"#,
    );

    let snapshot = scan_user_items_with_installation(directory.path(), |client| {
        client == AgentClient::CodexCli
    });
    let skill = snapshot.clients[0]
        .items
        .iter()
        .find(|item| item.kind == AgentItemKind::Skill)
        .unwrap();

    assert_eq!(skill.source, AgentItemSource::Team);
    assert_eq!(skill.version.as_deref(), Some("v1.2.0"));
}
