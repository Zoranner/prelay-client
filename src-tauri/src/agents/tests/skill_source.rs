use tempfile::tempdir;

use super::{
    super::{
        scan_agent_items, uninstall_user_item_with_installation, AgentClient, AgentItemKind,
        AgentItemSource,
    },
    write,
};

#[test]
fn marks_codex_system_skills_as_system() {
    let directory = tempdir().unwrap();
    write(
        directory
            .path()
            .join(".codex")
            .join("skills")
            .join(".system")
            .join("imagegen")
            .join("SKILL.md"),
        "# Imagegen\n",
    );
    write(
        directory
            .path()
            .join(".codex")
            .join("skills")
            .join("manual")
            .join("SKILL.md"),
        "# Manual\n",
    );
    write(
        directory
            .path()
            .join(".agents")
            .join("skills")
            .join(".system")
            .join("other")
            .join("SKILL.md"),
        "# Other\n",
    );

    let snapshot = scan_agent_items(directory.path(), AgentClient::CodexCli);
    let source = |name: &str| {
        snapshot
            .items
            .iter()
            .find(|item| item.kind == AgentItemKind::Skill && item.name == name)
            .unwrap()
            .source
    };

    assert_eq!(source("imagegen"), AgentItemSource::System);
    assert_eq!(
        serde_json::to_string(&source("imagegen")).unwrap(),
        "\"system\""
    );
    assert_eq!(source("manual"), AgentItemSource::Personal);
    assert_eq!(source("other"), AgentItemSource::Personal);
}

#[test]
fn refuses_to_uninstall_codex_system_skills() {
    let directory = tempdir().unwrap();
    let skill_file = directory
        .path()
        .join(".codex")
        .join("skills")
        .join(".system")
        .join("imagegen")
        .join("SKILL.md");
    write(&skill_file, "# Imagegen\n");

    let item = scan_agent_items(directory.path(), AgentClient::CodexCli)
        .items
        .into_iter()
        .find(|item| item.kind == AgentItemKind::Skill && item.name == "imagegen")
        .unwrap();

    let error = uninstall_user_item_with_installation(
        directory.path(),
        AgentClient::CodexCli,
        item.kind,
        &item.name,
        &item.source_path,
        |client| client == AgentClient::CodexCli,
    )
    .unwrap_err();

    assert_eq!(error, "内置技能不可卸载。");
    assert!(skill_file.exists());
}
