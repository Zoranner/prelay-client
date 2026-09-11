use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionFile;
use serde::{Deserialize, Serialize};

use crate::{agents::AgentClient, relay::client::ClientError};

use super::{atomic_write, decode_extension_file};

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const RULE_PACKAGE_STATE_FILE: &str = "rule.json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledRulePackage {
    package: String,
    version: String,
    commit_sha: String,
}

pub(crate) struct RuleInstallationStatus {
    pub action: RuleInstallAction,
    pub clients: Vec<AgentClient>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuleInstallAction {
    Install,
    Partial,
    Update,
    Installed,
}

pub(super) fn install_rule(
    target: &Path,
    package: &str,
    version: &str,
    commit_sha: &str,
    file: &ExtensionFile,
) -> Result<(), ClientError> {
    let content = decode_extension_file(file)?;
    std::str::from_utf8(&content)
        .map_err(|_| ClientError::new("invalid_response", "extension rule content is not UTF-8"))?;
    atomic_write(target, &content)?;
    write_rule_package_state(
        target,
        &InstalledRulePackage {
            package: package.to_string(),
            version: version.to_string(),
            commit_sha: commit_sha.to_string(),
        },
    )
}

pub(crate) fn rule_installation_status(
    targets: &[(AgentClient, PathBuf)],
    package: &str,
    version: &str,
    commit_sha: &str,
) -> Result<RuleInstallationStatus, ClientError> {
    let mut states = BTreeMap::new();
    for (_, target) in targets {
        if !states.contains_key(target) {
            states.insert(target.clone(), read_rule_package_state(target)?);
        }
    }
    let mut clients = Vec::new();
    let mut missing = false;
    let mut outdated = false;
    for (client, target) in targets {
        let state = states.get(target).expect("rule state was loaded");
        let Some(state) = state else {
            missing = true;
            continue;
        };
        if state.package != package {
            missing = true;
            continue;
        }
        clients.push(*client);
        if state.version != version || state.commit_sha != commit_sha {
            outdated = true;
        }
    }
    let action = if outdated {
        RuleInstallAction::Update
    } else if !clients.is_empty() && missing {
        RuleInstallAction::Partial
    } else if clients.is_empty() {
        RuleInstallAction::Install
    } else {
        RuleInstallAction::Installed
    };
    Ok(RuleInstallationStatus { action, clients })
}

pub(crate) fn retain_listed_rule_packages(
    targets: &[(AgentClient, PathBuf)],
    listed_packages: &BTreeSet<String>,
) -> Result<(), ClientError> {
    let unique_targets = targets
        .iter()
        .map(|(_, target)| target)
        .collect::<BTreeSet<_>>();
    for target in unique_targets {
        let Some(state) = read_rule_package_state(target)? else {
            continue;
        };
        if !listed_packages.contains(&state.package) {
            clear_rule_package_state(target)
                .map_err(|error| ClientError::new("local_extensions_error", error))?;
        }
    }
    Ok(())
}

pub(crate) fn clear_rule_package_state(target: &Path) -> Result<(), String> {
    let path = rule_package_state_path(target).ok_or_else(|| "规则目标没有父目录。".to_string())?;
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("无法清除 Rule 状态：{error}")),
    }
}

fn read_rule_package_state(target: &Path) -> Result<Option<InstalledRulePackage>, ClientError> {
    let path = rule_package_state_path(target).ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "rule target has no parent directory",
        )
    })?;
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents)
            .map(Some)
            .map_err(|error| {
                ClientError::new(
                    "local_extensions_error",
                    format!("无法读取 Rule 状态：{error}"),
                )
            }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ClientError::new(
            "local_extensions_error",
            format!("无法读取 Rule 状态：{error}"),
        )),
    }
}

fn write_rule_package_state(
    target: &Path,
    state: &InstalledRulePackage,
) -> Result<(), ClientError> {
    let path = rule_package_state_path(target).ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "rule target has no parent directory",
        )
    })?;
    let contents = serde_json::to_vec(state).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存 Rule 状态：{error}"),
        )
    })?;
    atomic_write(&path, &contents)
}

fn rule_package_state_path(target: &Path) -> Option<PathBuf> {
    target.parent().map(|parent| {
        parent
            .join(PRELAY_STATE_DIRECTORY)
            .join(RULE_PACKAGE_STATE_FILE)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use prelay_protocol::ExtensionFile;
    use tempfile::tempdir;

    use crate::agents::AgentClient;

    use super::{install_rule, rule_installation_status, RuleInstallAction};

    #[test]
    fn replaces_the_complete_rule_document() {
        let directory = tempdir().unwrap();
        let target = directory.path().join("AGENTS.md");
        fs::write(&target, "# Existing instructions\nKeep this").unwrap();

        install_rule(
            &target,
            "development-rules",
            "v1.0.0",
            "commit",
            &ExtensionFile {
                path: "AGENTS.md".to_string(),
                content_base64: BASE64.encode("# Published instructions"),
            },
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(target).unwrap(),
            "# Published instructions"
        );
        let status = rule_installation_status(
            &[(AgentClient::CodexCli, directory.path().join("AGENTS.md"))],
            "development-rules",
            "v1.0.0",
            "commit",
        )
        .unwrap();
        assert_eq!(status.action, RuleInstallAction::Installed);
    }

    #[test]
    fn rejects_rule_content_that_is_not_utf8() {
        let directory = tempdir().unwrap();
        let target = directory.path().join("AGENTS.md");

        let result = install_rule(
            &target,
            "development-rules",
            "v1.0.0",
            "commit",
            &ExtensionFile {
                path: "AGENTS.md".to_string(),
                content_base64: BASE64.encode([0xff]),
            },
        );

        assert!(result.is_err());
        assert!(!target.exists());
    }

    #[test]
    fn reports_partial_installation_when_a_detected_rule_target_has_no_state() {
        let directory = tempdir().unwrap();
        let codex = directory.path().join("codex").join("AGENTS.md");
        let claude = directory.path().join("claude").join("CLAUDE.md");

        install_rule(
            &codex,
            "development-rules",
            "v1.0.0",
            "commit",
            &ExtensionFile {
                path: "AGENTS.md".to_string(),
                content_base64: BASE64.encode("# Published instructions"),
            },
        )
        .unwrap();

        let status = rule_installation_status(
            &[
                (AgentClient::CodexCli, codex),
                (AgentClient::ClaudeCode, claude),
            ],
            "development-rules",
            "v1.0.0",
            "commit",
        )
        .unwrap();

        assert_eq!(status.action, RuleInstallAction::Partial);
        assert_eq!(status.clients, vec![AgentClient::CodexCli]);
    }
}
