use std::{fs, io::Write, path::Path};

use atomic_write_file::AtomicWriteFile;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::{ExtensionFile, ExtensionInstallBundle, ExtensionSummary};
use serde::{Deserialize, Serialize};

use crate::{
    agents::{
        agent_rule_targets, agent_rule_targets_with_clients, agent_skill_target_roots,
        agent_skill_targets, installed_agent_clients, AgentClient,
    },
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

pub use prelay_protocol::ExtensionKind;

pub(crate) mod mcp;
pub(crate) mod rules;
pub(crate) mod skills;

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
    #[serde(default)]
    pub install_action: ExtensionInstallAction,
    #[serde(default)]
    pub installed_clients: Vec<AgentClient>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionInstallAction {
    #[default]
    Install,
    Partial,
    Update,
    Installed,
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
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInstallResult {
    pub message: String,
}

pub async fn list_extensions(
    home: &Path,
    state: &NativeState,
    kind: ExtensionKind,
) -> Result<ExtensionCatalogSnapshot, ClientError> {
    let path = match kind {
        ExtensionKind::Rule => "/api/extensions/rules",
        ExtensionKind::Skill => "/api/extensions/skills",
        ExtensionKind::Mcp => "/api/extensions/mcp",
    };
    let client = authenticated_api(state).await?;
    let summaries: Vec<ExtensionSummary> = client.get(path).await?;
    let mut packages = summaries
        .into_iter()
        .map(|summary| ExtensionPackage {
            name: summary.name,
            repository: summary.repository,
            commit_sha: summary.latest.commit_sha,
            version: summary.latest.tag,
            kind,
            install_action: ExtensionInstallAction::Install,
            installed_clients: Vec::new(),
        })
        .collect::<Vec<_>>();
    let clients = installed_agent_clients();
    match kind {
        ExtensionKind::Rule => {
            let targets = agent_rule_targets_with_clients(&clients, home);
            let listed = packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            rules::retain_listed_rule_packages(&targets, &listed)?;
            for package in &mut packages {
                let status = rules::rule_installation_status(
                    &targets,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )?;
                package.install_action = match status.action {
                    rules::RuleInstallAction::Install => ExtensionInstallAction::Install,
                    rules::RuleInstallAction::Partial => ExtensionInstallAction::Partial,
                    rules::RuleInstallAction::Update => ExtensionInstallAction::Update,
                    rules::RuleInstallAction::Installed => ExtensionInstallAction::Installed,
                };
                package.installed_clients = status.clients;
            }
        }
        ExtensionKind::Skill => {
            let targets = agent_skill_targets(&clients, home);
            let target_roots = agent_skill_target_roots(&clients, home);
            let listed = packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            skills::retain_listed_skill_packages(&target_roots, &listed)?;
            for package in &mut packages {
                let status = skills::skill_installation_status(
                    &targets,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )?;
                package.install_action = match status.action {
                    skills::SkillInstallAction::Install => ExtensionInstallAction::Install,
                    skills::SkillInstallAction::Partial => ExtensionInstallAction::Partial,
                    skills::SkillInstallAction::Update => ExtensionInstallAction::Update,
                    skills::SkillInstallAction::Installed => ExtensionInstallAction::Installed,
                };
                package.installed_clients = status.clients;
            }
        }
        ExtensionKind::Mcp => {
            let clients = mcp_agent_clients(&clients);
            let listed = packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            mcp::retain_listed_mcp_packages(home, &clients, &listed)?;
            for package in &mut packages {
                let status = mcp::mcp_installation_status(
                    home,
                    &clients,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )?;
                package.install_action = match status.action {
                    mcp::McpInstallAction::Install => ExtensionInstallAction::Install,
                    mcp::McpInstallAction::Partial => ExtensionInstallAction::Partial,
                    mcp::McpInstallAction::Update => ExtensionInstallAction::Update,
                    mcp::McpInstallAction::Installed => ExtensionInstallAction::Installed,
                };
                package.installed_clients = status.clients;
            }
        }
    }
    Ok(ExtensionCatalogSnapshot { packages })
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

pub async fn read_mcp_install_manifest(
    state: &NativeState,
    package: &ExtensionPackage,
) -> Result<prelay_protocol::ExtensionMcpManifest, ClientError> {
    if package.kind != ExtensionKind::Mcp {
        return Err(ClientError::new(
            "invalid_request",
            "只有 MCP 扩展可以读取安装配置。",
        ));
    }
    let client = authenticated_api(state).await?;
    let bundle: ExtensionInstallBundle = client
        .get(&format!(
            "/api/extensions/{}/versions/{}/install",
            package.name, package.version
        ))
        .await?;
    validate_bundle(&bundle)?;
    if bundle.kind != ExtensionKind::Mcp {
        return Err(ClientError::new(
            "invalid_response",
            "扩展安装包类型不匹配。",
        ));
    }
    mcp::read_mcp_manifest(bundle.files.first().expect("validated MCP install bundle"))
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
                rules::install_rule(
                    &target,
                    &bundle.name,
                    &bundle.version.tag,
                    &bundle.version.commit_sha,
                    rules,
                )?;
            }
        }
        ExtensionKind::Skill => {
            for target_root in agent_skill_target_roots(&request.clients, home) {
                skills::install_skill_files(
                    &target_root,
                    &bundle.name,
                    &bundle.version.tag,
                    &bundle.version.commit_sha,
                    &bundle.files,
                    request.overwrite,
                )?;
            }
        }
        ExtensionKind::Mcp => {
            let manifest = mcp::read_mcp_manifest(
                bundle.files.first().expect("validated MCP install bundle"),
            )?;
            mcp::install_mcp(
                home,
                &request.clients,
                &bundle.name,
                &bundle.version.tag,
                &bundle.version.commit_sha,
                &manifest,
                request.overwrite,
            )?;
        }
    }
    Ok(ExtensionInstallResult {
        message: format!("已安装{}。", bundle.name),
    })
}

pub async fn update_all_skill_extensions(
    home: &Path,
    state: &NativeState,
) -> Result<ExtensionInstallResult, ClientError> {
    let catalog = list_extensions(home, state, ExtensionKind::Skill).await?;
    let clients = installed_agent_clients();
    let target_roots = agent_skill_target_roots(&clients, home);
    let targets = skills::outdated_skill_package_targets(&target_roots, &catalog.packages)?;
    if targets.is_empty() {
        return Ok(ExtensionInstallResult {
            message: "没有可更新扩展。".to_string(),
        });
    }

    let client = authenticated_api(state).await?;
    for (name, target_roots) in &targets {
        let package = catalog
            .packages
            .iter()
            .find(|package| package.name == *name)
            .expect("outdated package exists in the catalog");
        let bundle: ExtensionInstallBundle = client
            .get(&format!(
                "/api/extensions/{}/versions/{}/install",
                package.name, package.version
            ))
            .await?;
        validate_bundle(&bundle)?;
        for target_root in target_roots {
            skills::install_skill_files(
                target_root,
                &bundle.name,
                &bundle.version.tag,
                &bundle.version.commit_sha,
                &bundle.files,
                false,
            )?;
        }
    }
    Ok(ExtensionInstallResult {
        message: format!("已更新 {} 个扩展。", targets.len()),
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
        ExtensionKind::Mcp if bundle.files.len() == 1 => {
            mcp::read_mcp_manifest(&bundle.files[0]).map(|_| ())
        }
        _ => Err(ClientError::new(
            "invalid_response",
            "extension install bundle is invalid",
        )),
    }
}

fn mcp_agent_clients(clients: &[AgentClient]) -> Vec<AgentClient> {
    clients
        .iter()
        .copied()
        .filter(|client| {
            matches!(
                client,
                AgentClient::CodexCli | AgentClient::ClaudeCode | AgentClient::OpenCode
            )
        })
        .collect()
}

fn safe_skill_path(path: &str) -> bool {
    path.starts_with(SKILLS_PREFIX)
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

pub(super) fn decode_extension_file(file: &ExtensionFile) -> Result<Vec<u8>, ClientError> {
    BASE64.decode(&file.content_base64).map_err(|_| {
        ClientError::new(
            "invalid_response",
            "extension install bundle contains invalid Base64 content",
        )
    })
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
