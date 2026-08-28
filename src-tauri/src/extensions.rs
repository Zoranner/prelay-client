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
const README_PATH: &str = "README.md";
const RULES_PATH: &str = "AGENTS.md";
const CODEX_PLUGIN_PATH: &str = ".codex-plugin/plugin.json";
const OPEN_CODE_PLUGIN_ROOT: &str = ".opencode/plugins/";
const MCP_SERVER_PATH: &str = "server.json";

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

#[derive(Debug, Deserialize)]
struct GiteaRepository {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GiteaCommit {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GiteaTag {
    name: String,
    commit: GiteaCommit,
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

pub async fn install_extension(
    home: &Path,
    request: &ExtensionInstallRequest,
) -> Result<ExtensionInstallResult, String> {
    let client = ExtensionCatalogClient::new(Client::new());
    let package = client.resolve_package(&request.package).await?;
    let paths = client
        .package_paths(&package.repository, &package.commit_sha)
        .await?;
    validate_install(&package, &paths)?;

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
        message: format!("已安装{}。", package.repository),
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
                let tag = match self.latest_release_tag(&repository.name).await {
                    Ok(tag) => tag,
                    Err(_) => continue,
                };
                let paths = match self.package_paths(&repository.name, &tag.commit.sha).await {
                    Ok(paths) => paths,
                    Err(_) => continue,
                };
                if let Some(package) =
                    package_from_paths(repository.name, tag.commit.sha, tag.name, &paths)
                {
                    packages.push(package);
                }
            }
            if count < 50 {
                break;
            }
            page += 1;
        }
        packages.sort_by(|left, right| left.repository.cmp(&right.repository));
        Ok(ExtensionCatalogSnapshot { packages })
    }

    async fn resolve_package(
        &self,
        package: &ExtensionPackage,
    ) -> Result<ExtensionPackage, String> {
        if !valid_repository_name(&package.repository) || !valid_commit_sha(&package.commit_sha) {
            return Err("扩展标识无效。".to_string());
        }
        if !valid_release_tag(&package.version) {
            return Err("扩展版本无效。".to_string());
        }
        let tag = self.tag(&package.repository, &package.version).await?;
        if tag.commit.sha != package.commit_sha {
            return Err("扩展版本与提交不匹配。".to_string());
        }
        let paths = self
            .package_paths(&package.repository, &package.commit_sha)
            .await?;
        package_from_paths(
            package.repository.clone(),
            package.commit_sha.clone(),
            package.version.clone(),
            &paths,
        )
        .ok_or_else(|| "扩展仓库不符合扩展规范。".to_string())
    }

    async fn latest_release_tag(&self, repository: &str) -> Result<GiteaTag, String> {
        let tags: Vec<GiteaTag> = self
            .get_json(
                &extension_tags_url(repository),
                &[("limit", "50"), ("page", "1")],
            )
            .await?;
        tags.into_iter()
            .find(|tag| valid_release_tag(&tag.name) && valid_commit_sha(&tag.commit.sha))
            .ok_or_else(|| "扩展仓库没有可用的发布 tag。".to_string())
    }

    async fn tag(&self, repository: &str, version: &str) -> Result<GiteaTag, String> {
        let tags: Vec<GiteaTag> = self
            .get_json(
                &extension_tags_url(repository),
                &[("limit", "50"), ("page", "1")],
            )
            .await?;
        tags.into_iter()
            .find(|tag| tag.name == version && valid_commit_sha(&tag.commit.sha))
            .ok_or_else(|| "扩展版本不存在。".to_string())
    }

    async fn read_readme(&self, package: &ExtensionPackage) -> Result<String, String> {
        let package = self.resolve_package(package).await?;
        self.read_file(&package.repository, &package.commit_sha, README_PATH)
            .await
    }

    async fn package_paths(
        &self,
        repository: &str,
        commit_sha: &str,
    ) -> Result<Vec<String>, String> {
        let tree: GiteaTree = self
            .get_json(
                &format!(
                    "{API_BASE_URL}/repos/{ORGANIZATION}/{}/git/trees/{}",
                    repository, commit_sha
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

fn extension_tags_url(repository: &str) -> String {
    format!("{API_BASE_URL}/repos/{ORGANIZATION}/{repository}/tags")
}

fn package_from_paths(
    repository: String,
    commit_sha: String,
    version: String,
    paths: &[String],
) -> Option<ExtensionPackage> {
    (valid_repository_name(&repository)
        && valid_commit_sha(&commit_sha)
        && valid_release_tag(&version)
        && paths.iter().any(|path| path == README_PATH))
    .then_some(ExtensionPackage {
        repository,
        commit_sha,
        version,
        kind: extension_kind(paths)?,
    })
}

fn extension_kind(paths: &[String]) -> Option<ExtensionKind> {
    if paths.iter().any(|path| path == CODEX_PLUGIN_PATH)
        || paths.iter().any(|path| {
            path.starts_with(OPEN_CODE_PLUGIN_ROOT)
                && matches!(path.rsplit('.').next(), Some("js" | "ts"))
        })
    {
        return Some(ExtensionKind::Plugin);
    }
    if paths.iter().any(|path| path == MCP_SERVER_PATH) {
        return Some(ExtensionKind::Mcp);
    }
    if paths
        .iter()
        .any(|path| path.starts_with("skills/") && path.ends_with("/SKILL.md"))
    {
        return Some(ExtensionKind::Skill);
    }
    paths
        .iter()
        .any(|path| path == RULES_PATH)
        .then_some(ExtensionKind::Rule)
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

fn valid_release_tag(tag: &str) -> bool {
    let Some(version) = tag.strip_prefix('v') else {
        return false;
    };
    let parts: Vec<_> = version
        .split('-')
        .next()
        .unwrap_or_default()
        .split('.')
        .collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty() && part.bytes().all(|character| character.is_ascii_digit())
        })
}

fn validate_install(package: &ExtensionPackage, paths: &[String]) -> Result<(), String> {
    match package.kind {
        ExtensionKind::Rule if paths.iter().any(|path| path == RULES_PATH) => Ok(()),
        ExtensionKind::Skill if !skill_paths(paths).is_empty() => Ok(()),
        ExtensionKind::Plugin | ExtensionKind::Mcp => {
            Err("插件和 MCP 的官方配置合并契约尚未定义，当前仅支持查看详情。".to_string())
        }
        _ => Err("扩展包缺少可安装的入口文件。".to_string()),
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
        extension_tags_url, merge_managed_rule, package_from_paths, rule_targets,
        skill_target_roots, ExtensionKind,
    };
    use crate::agents::AgentClient;

    #[test]
    fn resolves_release_tags_through_gitea_tags_endpoint() {
        assert_eq!(
            extension_tags_url("development-rules"),
            "https://git.kimo.ink/api/v1/repos/agents/development-rules/tags"
        );
    }

    #[test]
    fn catalog_infers_skill_packages_from_standard_paths() {
        let package = package_from_paths(
            "engineering-review".to_string(),
            "a".repeat(40),
            "v0.1.0".to_string(),
            &[
                "README.md".to_string(),
                "skills/review-engineering/SKILL.md".to_string(),
            ],
        );

        assert_eq!(package.unwrap().kind, ExtensionKind::Skill);
    }

    #[test]
    fn catalog_prefers_plugins_over_the_skills_they_contain() {
        let package = package_from_paths(
            "superpowers".to_string(),
            "b".repeat(40),
            "v6.3.0-prelay.1".to_string(),
            &[
                "README.md".to_string(),
                ".codex-plugin/plugin.json".to_string(),
                "skills/brainstorming/SKILL.md".to_string(),
            ],
        );

        assert_eq!(package.unwrap().kind, ExtensionKind::Plugin);
    }

    #[test]
    fn catalog_ignores_repositories_without_a_standard_extension_entry() {
        let package = package_from_paths(
            "ordinary-project".to_string(),
            "c".repeat(40),
            "v0.1.0".to_string(),
            &["README.md".to_string(), "src/main.rs".to_string()],
        );

        assert!(package.is_none());
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
