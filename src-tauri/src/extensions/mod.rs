use std::{fs, io::Write, path::Path};

use atomic_write_file::AtomicWriteFile;
use prelay_protocol::{ExtensionInstallBundle, ExtensionSummary};
use serde::{Deserialize, Serialize};

use crate::{
    agents::{agent_rule_targets, agent_skill_target_roots, AgentClient},
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

pub use prelay_protocol::ExtensionKind;

const RULES_PATH: &str = "AGENTS.md";
const SKILLS_PREFIX: &str = "skills/";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionPackage {
    pub name: String,
    pub repository: String,
    pub commit_sha: String,
    pub version: String,
    pub kind: ExtensionKind,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionCatalogSnapshot {
    pub packages: Vec<ExtensionPackage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInstallRequest {
    pub package: ExtensionPackage,
    pub clients: Vec<AgentClient>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInstallResult {
    pub message: String,
}

pub async fn list_extensions(
    state: &NativeState,
    kind: ExtensionKind,
) -> Result<ExtensionCatalogSnapshot, ClientError> {
    let path = match kind {
        ExtensionKind::Rule => "/api/extensions/rules",
        ExtensionKind::Skill => "/api/extensions/skills",
        ExtensionKind::Plugin => "/api/extensions/plugins",
        ExtensionKind::Mcp => "/api/extensions/mcp",
    };
    let client = authenticated_api(state).await?;
    let summaries: Vec<ExtensionSummary> = client.get(path).await?;
    Ok(ExtensionCatalogSnapshot {
        packages: summaries
            .into_iter()
            .map(|summary| ExtensionPackage {
                name: summary.name,
                repository: summary.repository,
                commit_sha: summary.latest.commit_sha,
                version: summary.latest.tag,
                kind,
            })
            .collect(),
    })
}

pub async fn read_extension_readme(
    state: &NativeState,
    package: &ExtensionPackage,
) -> Result<String, ClientError> {
    let client = authenticated_api(state).await?;
    let bytes = client
        .get_bytes(&format!(
            "/api/extensions/{}/versions/{}/readme",
            package.name, package.version
        ))
        .await?;
    String::from_utf8(bytes)
        .map_err(|_| ClientError::new("invalid_response", "extension README is not UTF-8"))
}

pub async fn install_extension(
    home: &Path,
    state: &NativeState,
    request: &ExtensionInstallRequest,
) -> Result<ExtensionInstallResult, ClientError> {
    let client = authenticated_api(state).await?;
    let bundle: ExtensionInstallBundle = client
        .get(&format!(
            "/api/extensions/{}/versions/{}/install",
            request.package.name, request.package.version
        ))
        .await?;
    validate_bundle(&bundle)?;

    match bundle.kind {
        ExtensionKind::Rule => {
            let rules = bundle.files.first().expect("validated rule bundle");
            for target in agent_rule_targets(&request.clients, home) {
                let current = fs::read_to_string(&target).unwrap_or_default();
                atomic_write(
                    &target,
                    merge_managed_rule(&current, &bundle.name, &rules.content).as_bytes(),
                )?;
            }
        }
        ExtensionKind::Skill => {
            for source in bundle.files {
                let relative = source
                    .path
                    .strip_prefix(SKILLS_PREFIX)
                    .expect("validated skill path");
                for target_root in agent_skill_target_roots(&request.clients, home) {
                    atomic_write(&target_root.join(relative), source.content.as_bytes())?;
                }
            }
        }
        ExtensionKind::Plugin | ExtensionKind::Mcp => unreachable!("validated install bundle"),
    }
    Ok(ExtensionInstallResult {
        message: format!("已安装{}。", bundle.name),
    })
}

fn validate_bundle(bundle: &ExtensionInstallBundle) -> Result<(), ClientError> {
    match bundle.kind {
        ExtensionKind::Rule if bundle.files.len() == 1 && bundle.files[0].path == RULES_PATH => {
            Ok(())
        }
        ExtensionKind::Skill
            if !bundle.files.is_empty()
                && bundle.files.iter().all(|file| safe_skill_path(&file.path)) =>
        {
            Ok(())
        }
        ExtensionKind::Plugin | ExtensionKind::Mcp => Err(ClientError::new(
            "extension_install_unsupported",
            "插件和 MCP 当前仅支持查看详情。",
        )),
        _ => Err(ClientError::new(
            "invalid_response",
            "extension install bundle is invalid",
        )),
    }
}

fn safe_skill_path(path: &str) -> bool {
    path.starts_with(SKILLS_PREFIX)
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

fn merge_managed_rule(existing: &str, package: &str, contents: &str) -> String {
    let start = format!("<!-- prelay-extension:{package}:start -->");
    let end = format!("<!-- prelay-extension:{package}:end -->");
    let block = format!("{start}\n{contents}\n{end}");
    let mut merged = existing.to_string();
    if let Some(start_index) = merged.find(&start) {
        if let Some(end_offset) = merged[start_index..].find(&end) {
            merged.replace_range(start_index..start_index + end_offset + end.len(), &block);
            return merged;
        }
    }
    if !merged.trim().is_empty() {
        merged.push_str("\n\n");
    }
    merged.push_str(&block);
    merged
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), ClientError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(storage_error)?;
    }
    let mut file = AtomicWriteFile::open(path).map_err(storage_error)?;
    file.write_all(contents).map_err(storage_error)?;
    file.commit().map_err(storage_error)
}

fn storage_error(error: std::io::Error) -> ClientError {
    ClientError::new(
        "local_extensions_error",
        format!("无法写入扩展文件：{error}"),
    )
}
