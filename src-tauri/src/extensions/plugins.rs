use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionInstallBundle;
use serde::Serialize;

use crate::{
    agents::{register_codex_plugin, AgentClient},
    relay::client::ClientError,
};

use super::atomic_write;

const CODEX_PLUGIN_MANIFEST: &str = ".codex-plugin/plugin.json";
const OPENCODE_PLUGIN_PREFIX: &str = ".opencode/plugins/";
const PRELAY_MARKETPLACE: &str = "prelay";
const MANAGED_PLUGINS_DIRECTORY: &str = ".prelay/plugins";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManagedPluginPackage {
    package: String,
    version: String,
    commit_sha: String,
    files: BTreeSet<String>,
}

pub(super) fn install_plugin(
    home: &Path,
    bundle: &ExtensionInstallBundle,
    clients: &[AgentClient],
) -> Result<(), ClientError> {
    if clients
        .iter()
        .any(|client| matches!(client, AgentClient::CodexCli | AgentClient::ChatGpt))
    {
        install_codex_plugin(home, bundle)?;
    }
    if clients
        .iter()
        .any(|client| matches!(client, AgentClient::OpenCode))
    {
        install_opencode_plugin(home, bundle)?;
    }
    Ok(())
}

fn install_codex_plugin(home: &Path, bundle: &ExtensionInstallBundle) -> Result<(), ClientError> {
    if !bundle
        .files
        .iter()
        .any(|file| file.path == CODEX_PLUGIN_MANIFEST)
    {
        return Err(ClientError::new(
            "invalid_response",
            "插件不包含 Codex 插件清单。",
        ));
    }
    let version = safe_component(&bundle.version.tag)?;
    let name = safe_component(&bundle.name)?;
    let destination = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join(PRELAY_MARKETPLACE)
        .join(name)
        .join(version);
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(storage_error)?;
    }
    for file in &bundle.files {
        if safe_path(&file.path) {
            atomic_write(&destination.join(&file.path), file.content.as_bytes())?;
        }
    }
    register_codex_plugin(home, &format!("{}@{PRELAY_MARKETPLACE}", bundle.name))
        .map_err(local_error)
}

fn install_opencode_plugin(
    home: &Path,
    bundle: &ExtensionInstallBundle,
) -> Result<(), ClientError> {
    let files = bundle
        .files
        .iter()
        .filter(|file| {
            file.path.starts_with(OPENCODE_PLUGIN_PREFIX)
                && (file.path.ends_with(".js") || file.path.ends_with(".ts"))
                && safe_path(&file.path)
        })
        .collect::<Vec<_>>();
    if files.is_empty() {
        return Err(ClientError::new(
            "invalid_response",
            "插件不包含 OpenCode 本地插件文件。",
        ));
    }
    let root = home.join(".config").join("opencode").join("plugins");
    for file in &files {
        let relative = file
            .path
            .strip_prefix(OPENCODE_PLUGIN_PREFIX)
            .expect("validated OpenCode plugin path");
        atomic_write(&root.join(relative), file.content.as_bytes())?;
    }
    let files = files
        .iter()
        .filter_map(|file| file.path.strip_prefix(OPENCODE_PLUGIN_PREFIX))
        .map(ToString::to_string)
        .collect();
    let manifest = ManagedPluginPackage {
        package: bundle.name.clone(),
        version: bundle.version.tag.clone(),
        commit_sha: bundle.version.commit_sha.clone(),
        files,
    };
    let manifest_path = plugin_manifest_path(home, &bundle.name)?;
    let contents = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| local_error(format!("无法保存插件安装记录：{error}")))?;
    atomic_write(&manifest_path, &contents)?;
    Ok(())
}

fn plugin_manifest_path(home: &Path, package: &str) -> Result<PathBuf, ClientError> {
    safe_component(package)?;
    let key = package
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(home
        .join(MANAGED_PLUGINS_DIRECTORY)
        .join(format!("{key}.json")))
}

pub(super) fn valid_plugin_bundle(bundle: &ExtensionInstallBundle) -> bool {
    !bundle.files.is_empty() && bundle.files.iter().all(|file| safe_path(&file.path))
}

fn safe_component(value: &str) -> Result<&str, ClientError> {
    if !value.is_empty() && !value.contains(['/', '\\']) && value != "." && value != ".." {
        return Ok(value);
    }
    Err(ClientError::new("invalid_response", "插件包标识无效。"))
}

fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

fn storage_error(error: std::io::Error) -> ClientError {
    ClientError::new(
        "local_extensions_error",
        format!("无法写入插件文件：{error}"),
    )
}

fn local_error(message: String) -> ClientError {
    ClientError::new("local_extensions_error", message)
}

#[cfg(test)]
mod tests {
    use prelay_protocol::{ExtensionFile, ExtensionInstallBundle, ExtensionKind, ExtensionVersion};
    use tempfile::tempdir;

    use super::install_opencode_plugin;

    #[test]
    fn installs_opencode_plugin_into_the_global_plugin_directory() {
        let directory = tempdir().unwrap();
        let bundle = ExtensionInstallBundle {
            name: "review-tools".to_string(),
            kind: ExtensionKind::Plugin,
            version: ExtensionVersion {
                tag: "v1.0.0".to_string(),
                commit_sha: "a".repeat(40),
                updated_at: "2026-08-29T00:00:00Z".to_string(),
            },
            files: vec![ExtensionFile {
                path: ".opencode/plugins/review.ts".to_string(),
                content: "export const Review = async () => ({});".to_string(),
            }],
        };

        install_opencode_plugin(directory.path(), &bundle).unwrap();

        assert_eq!(
            std::fs::read_to_string(directory.path().join(".config/opencode/plugins/review.ts"),)
                .unwrap(),
            "export const Review = async () => ({});"
        );
        let manifest = std::fs::read_to_string(
            directory
                .path()
                .join(".prelay/plugins/7265766965772d746f6f6c73.json"),
        )
        .unwrap();
        assert!(manifest.contains("\"version\": \"v1.0.0\""));
        assert!(manifest.contains("review.ts"));
    }
}
