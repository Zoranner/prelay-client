use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::agents::AgentClient;

const ORGANIZATION: &str = "agents";
const API_BASE_URL: &str = "https://git.kimo.ink/api/v1";
const MANIFEST_PATH: &str = ".prelay.json";
const README_PATH: &str = "README.md";
const RULES_PATH: &str = "AGENTS.md";

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionKind {
    Rule,
    Plugin,
    Mcp,
    Skill,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionPackage {
    pub repository: String,
    pub commit_sha: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub kind: ExtensionKind,
    pub risk: String,
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
pub struct ExtensionInstallPreview {
    pub supported: bool,
    pub message: Option<String>,
    pub actions: Vec<ExtensionInstallAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInstallAction {
    pub target: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInstallResult {
    pub message: String,
    pub actions: Vec<ExtensionInstallAction>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    schema: u32,
    version: String,
    name: String,
    summary: String,
    kind: ExtensionKind,
    #[serde(default = "default_risk")]
    risk: String,
}

#[derive(Debug, Deserialize)]
struct GiteaRepository {
    name: String,
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct GiteaCommit {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GiteaContent {
    content: String,
    encoding: String,
}

#[derive(Debug, Deserialize)]
struct GiteaTree {
    tree: Vec<GiteaTreeEntry>,
}

#[derive(Debug, Deserialize)]
struct GiteaTreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
}

fn default_risk() -> String {
    "files-only".to_string()
}

pub async fn list_extensions() -> Result<ExtensionCatalogSnapshot, String> {
    ExtensionCatalogClient::new(Client::new())
        .list_packages()
        .await
}

pub async fn read_extension_readme(package: &ExtensionPackage) -> Result<String, String> {
    ExtensionCatalogClient::new(Client::new())
        .read_readme(package)
        .await
}

pub async fn preview_extension_install(
    request: &ExtensionInstallRequest,
) -> Result<ExtensionInstallPreview, String> {
    let client = ExtensionCatalogClient::new(Client::new());
    let package = client.resolve_package(&request.package).await?;
    let paths = client.package_paths(&package).await?;
    Ok(install_preview(&package, &request.clients, &paths))
}

pub async fn install_extension(
    home: &Path,
    request: &ExtensionInstallRequest,
) -> Result<ExtensionInstallResult, String> {
    let client = ExtensionCatalogClient::new(Client::new());
    let package = client.resolve_package(&request.package).await?;
    let paths = client.package_paths(&package).await?;
    let preview = install_preview(&package, &request.clients, &paths);
    if !preview.supported {
        return Err(preview
            .message
            .unwrap_or_else(|| "此扩展暂不支持安装。".to_string()));
    }

    match package.kind {
        ExtensionKind::Rule => {
            let rules = client
                .read_file(&package.repository, &package.commit_sha, RULES_PATH)
                .await?;
            for target in rule_targets(&request.clients, home) {
                let current = fs::read_to_string(&target).unwrap_or_default();
                atomic_write(
                    &target,
                    merge_managed_rule(&current, &package.repository, &rules).as_bytes(),
                )?;
            }
        }
        ExtensionKind::Skill => {
            for source in skill_paths(&paths) {
                let contents = client
                    .read_file(&package.repository, &package.commit_sha, &source)
                    .await?;
                let relative = source
                    .strip_prefix("skills/")
                    .ok_or_else(|| "Skill 路径无效。".to_string())?;
                for target_root in skill_target_roots(&request.clients, home) {
                    atomic_write(&target_root.join(relative), contents.as_bytes())?;
                }
            }
        }
        ExtensionKind::Plugin | ExtensionKind::Mcp => unreachable!(),
    }

    Ok(ExtensionInstallResult {
        message: format!("已安装{}。", package.name),
        actions: preview.actions,
    })
}

struct ExtensionCatalogClient {
    http: Client,
}

impl ExtensionCatalogClient {
    fn new(http: Client) -> Self {
        Self { http }
    }

    async fn list_packages(&self) -> Result<ExtensionCatalogSnapshot, String> {
        let mut page = 1;
        let mut packages = Vec::new();
        loop {
            let repositories: Vec<GiteaRepository> = self
                .get_json(
                    &format!("{API_BASE_URL}/orgs/{ORGANIZATION}/repos"),
                    &[("limit", "50"), ("page", &page.to_string())],
                )
                .await?;
            let count = repositories.len();
            for repository in repositories {
                if !valid_repository_name(&repository.name) {
                    continue;
                }
                let commit = match self
                    .resolve_commit(&repository.name, &repository.default_branch)
                    .await
                {
                    Ok(commit) => commit,
                    Err(_) => continue,
                };
                let manifest = match self.read_manifest(&repository.name, &commit).await {
                    Ok(manifest) => manifest,
                    Err(_) => continue,
                };
                if let Some(package) = package_from_manifest(repository.name, commit, manifest) {
                    packages.push(package);
                }
            }
            if count < 50 {
                break;
            }
            page += 1;
        }
        packages.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(ExtensionCatalogSnapshot { packages })
    }

    async fn resolve_package(
        &self,
        package: &ExtensionPackage,
    ) -> Result<ExtensionPackage, String> {
        if !valid_repository_name(&package.repository) || !valid_commit_sha(&package.commit_sha) {
            return Err("扩展标识无效。".to_string());
        }
        let manifest = self
            .read_manifest(&package.repository, &package.commit_sha)
            .await?;
        package_from_manifest(
            package.repository.clone(),
            package.commit_sha.clone(),
            manifest,
        )
        .ok_or_else(|| "扩展清单无效。".to_string())
    }

    async fn resolve_commit(&self, repository: &str, branch: &str) -> Result<String, String> {
        let commit: GiteaCommit = self
            .get_json(
                &format!("{API_BASE_URL}/repos/{ORGANIZATION}/{repository}/commits/{branch}"),
                &[],
            )
            .await?;
        valid_commit_sha(&commit.sha)
            .then_some(commit.sha)
            .ok_or_else(|| "扩展仓库返回了无效提交。".to_string())
    }

    async fn read_manifest(&self, repository: &str, commit_sha: &str) -> Result<Manifest, String> {
        let content = self
            .read_file(repository, commit_sha, MANIFEST_PATH)
            .await?;
        serde_json::from_str(&content).map_err(|error| format!("扩展清单格式无效：{error}"))
    }

    async fn read_readme(&self, package: &ExtensionPackage) -> Result<String, String> {
        let package = self.resolve_package(package).await?;
        self.read_file(&package.repository, &package.commit_sha, README_PATH)
            .await
    }

    async fn package_paths(&self, package: &ExtensionPackage) -> Result<Vec<String>, String> {
        let tree: GiteaTree = self
            .get_json(
                &format!(
                    "{API_BASE_URL}/repos/{ORGANIZATION}/{}/git/trees/{}",
                    package.repository, package.commit_sha
                ),
                &[("recursive", "true")],
            )
            .await?;
        Ok(tree
            .tree
            .into_iter()
            .filter(|entry| entry.entry_type == "blob")
            .map(|entry| entry.path)
            .collect())
    }

    async fn read_file(
        &self,
        repository: &str,
        commit_sha: &str,
        path: &str,
    ) -> Result<String, String> {
        let content: GiteaContent = self
            .get_json(
                &format!("{API_BASE_URL}/repos/{ORGANIZATION}/{repository}/contents/{path}"),
                &[("ref", commit_sha)],
            )
            .await?;
        if content.encoding != "base64" {
            return Err("扩展文件编码不受支持。".to_string());
        }
        let encoded = content.content.replace(['\r', '\n'], "");
        let bytes = BASE64
            .decode(encoded)
            .map_err(|error| format!("扩展文件无法解码：{error}"))?;
        String::from_utf8(bytes).map_err(|error| format!("扩展文件不是 UTF-8 文本：{error}"))
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, &str)],
    ) -> Result<T, String> {
        let response = self
            .http
            .get(url)
            .query(query)
            .send()
            .await
            .map_err(|error| format!("无法读取扩展库：{error}"))?;
        let response = response
            .error_for_status()
            .map_err(|error| format!("无法读取扩展库：{error}"))?;
        response
            .json()
            .await
            .map_err(|error| format!("扩展库响应格式无效：{error}"))
    }
}

fn package_from_manifest(
    repository: String,
    commit_sha: String,
    manifest: Manifest,
) -> Option<ExtensionPackage> {
    (manifest.schema == 1
        && valid_repository_name(&repository)
        && valid_commit_sha(&commit_sha)
        && !manifest.version.trim().is_empty()
        && !manifest.name.trim().is_empty()
        && !manifest.summary.trim().is_empty())
    .then_some(ExtensionPackage {
        repository,
        commit_sha,
        name: manifest.name,
        version: manifest.version,
        summary: manifest.summary,
        kind: manifest.kind,
        risk: manifest.risk,
    })
}

fn valid_repository_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, b'-' | b'_'))
}

fn valid_commit_sha(sha: &str) -> bool {
    sha.len() == 40 && sha.bytes().all(|character| character.is_ascii_hexdigit())
}

fn install_preview(
    package: &ExtensionPackage,
    clients: &[AgentClient],
    paths: &[String],
) -> ExtensionInstallPreview {
    match package.kind {
        ExtensionKind::Rule if paths.iter().any(|path| path == RULES_PATH) => {
            ExtensionInstallPreview {
                supported: true,
                message: None,
                actions: rule_targets(clients, Path::new("~"))
                    .into_iter()
                    .map(|path| ExtensionInstallAction {
                        target: path.display().to_string(),
                        description: "合并扩展托管规则区块".to_string(),
                    })
                    .collect(),
            }
        }
        ExtensionKind::Skill if !skill_paths(paths).is_empty() => ExtensionInstallPreview {
            supported: true,
            message: None,
            actions: skill_target_roots(clients, Path::new("~"))
                .into_iter()
                .map(|path| ExtensionInstallAction {
                    target: path.display().to_string(),
                    description: format!("复制 {} 个 Skill 文件", skill_paths(paths).len()),
                })
                .collect(),
        },
        ExtensionKind::Plugin | ExtensionKind::Mcp => ExtensionInstallPreview {
            supported: false,
            message: Some(
                "插件和 MCP 的官方配置合并契约尚未定义，当前仅支持查看详情。".to_string(),
            ),
            actions: Vec::new(),
        },
        _ => ExtensionInstallPreview {
            supported: false,
            message: Some("扩展包缺少可安装的入口文件。".to_string()),
            actions: Vec::new(),
        },
    }
}

fn rule_targets(clients: &[AgentClient], home: &Path) -> Vec<PathBuf> {
    let mut targets = BTreeSet::new();
    for client in clients {
        let target = match client {
            AgentClient::CodexCli | AgentClient::ChatGpt => home.join(".codex").join("AGENTS.md"),
            AgentClient::ClaudeCode => home.join(".claude").join("CLAUDE.md"),
        };
        targets.insert(target);
    }
    targets.into_iter().collect()
}

fn skill_target_roots(clients: &[AgentClient], home: &Path) -> Vec<PathBuf> {
    let mut targets = BTreeSet::new();
    for client in clients {
        let target = match client {
            AgentClient::CodexCli | AgentClient::ChatGpt => home.join(".agents").join("skills"),
            AgentClient::ClaudeCode => home.join(".claude").join("skills"),
        };
        targets.insert(target);
    }
    targets.into_iter().collect()
}

fn skill_paths(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter(|path| path.starts_with("skills/") && !path.ends_with('/'))
        .cloned()
        .collect()
}

fn merge_managed_rule(existing: &str, package: &str, contents: &str) -> String {
    let start = format!("<!-- prelay-extension:{package}:start -->");
    let end = format!("<!-- prelay-extension:{package}:end -->");
    let block = format!("{start}\n{contents}\n{end}");
    let mut merged = existing.to_string();
    if let Some(start_index) = merged.find(&start) {
        if let Some(end_offset) = merged[start_index..].find(&end) {
            let end_index = start_index + end_offset + end.len();
            merged.replace_range(start_index..end_index, &block);
            return merged;
        }
    }
    if !merged.trim().is_empty() {
        merged.push_str("\n\n");
    }
    merged.push_str(&block);
    merged
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("无法创建扩展目录：{error}"))?;
    }
    let mut file =
        AtomicWriteFile::open(path).map_err(|error| format!("无法写入扩展文件：{error}"))?;
    file.write_all(contents)
        .map_err(|error| format!("无法写入扩展文件：{error}"))?;
    file.commit()
        .map_err(|error| format!("无法保存扩展文件：{error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        merge_managed_rule, package_from_manifest, rule_targets, skill_target_roots, ExtensionKind,
        Manifest,
    };
    use crate::agents::AgentClient;

    #[test]
    fn catalog_ignores_repositories_with_invalid_manifests() {
        let valid = package_from_manifest(
            "engineering-review".to_string(),
            "a".repeat(40),
            Manifest {
                schema: 1,
                version: "0.1.0".to_string(),
                name: "工程评审".to_string(),
                summary: "评审工程质量".to_string(),
                kind: ExtensionKind::Skill,
                risk: "files-only".to_string(),
            },
        );
        let invalid = package_from_manifest(
            "missing-manifest".to_string(),
            "b".repeat(40),
            Manifest {
                schema: 2,
                version: "0.1.0".to_string(),
                name: "无效".to_string(),
                summary: "无效".to_string(),
                kind: ExtensionKind::Skill,
                risk: "files-only".to_string(),
            },
        );

        assert_eq!(valid.unwrap().kind, ExtensionKind::Skill);
        assert!(invalid.is_none());
    }

    #[test]
    fn merges_one_managed_rule_block_without_replacing_user_rules() {
        let merged = merge_managed_rule("# 用户规则", "development-rules", "# 包规则");

        assert!(merged.contains("# 用户规则"));
        assert!(merged.contains("# 包规则"));
        assert_eq!(
            merged
                .matches("prelay-extension:development-rules:start")
                .count(),
            1
        );
    }

    #[test]
    fn codex_and_chatgpt_share_one_install_target() {
        let home = std::path::Path::new("home");
        assert_eq!(
            rule_targets(&[AgentClient::CodexCli, AgentClient::ChatGpt], home).len(),
            1
        );
        assert_eq!(
            skill_target_roots(&[AgentClient::CodexCli, AgentClient::ChatGpt], home).len(),
            1
        );
    }
}
