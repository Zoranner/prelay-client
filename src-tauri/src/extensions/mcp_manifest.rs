use std::path::{Path, PathBuf, MAIN_SEPARATOR, MAIN_SEPARATOR_STR};

use prelay_protocol::{
    validate_mcp_manifest as validate_manifest, ExtensionFile, ExtensionMcpManifest,
    ExtensionMcpTransport, McpManifestError,
};

use crate::relay::client::ClientError;

use super::decode_extension_file;

pub(super) fn read_mcp_manifest(file: &ExtensionFile) -> Result<ExtensionMcpManifest, ClientError> {
    if file.path != "server.json" {
        return Err(ClientError::new(
            "invalid_response",
            "MCP 安装包缺少 server.json。",
        ));
    }
    let content = decode_extension_file(file)?;
    let manifest = serde_json::from_slice::<ExtensionMcpManifest>(&content)
        .map_err(|_| ClientError::new("invalid_response", "MCP 清单不是有效的 JSON。"))?;
    validate_manifest(&manifest).map_err(manifest_error)?;
    Ok(manifest)
}

fn manifest_error(error: McpManifestError) -> ClientError {
    match error {
        McpManifestError::ServerName => ClientError::new("invalid_response", "MCP 服务名无效。"),
        McpManifestError::StdioTransport => {
            ClientError::new("invalid_response", "MCP 本地进程配置无效。")
        }
        McpManifestError::HttpTransport => {
            ClientError::new("invalid_response", "MCP HTTP 配置无效。")
        }
    }
}

pub(super) fn expand_manifest(
    manifest: &ExtensionMcpManifest,
    home: &Path,
    app_directory: &Path,
) -> Result<ExtensionMcpManifest, ClientError> {
    let mut expanded = manifest.clone();
    if let ExtensionMcpTransport::Stdio { command, .. } = &mut expanded.transport {
        for part in command.iter_mut() {
            *part = expand_user_directories(part, home, app_directory)?;
        }
    }
    Ok(expanded)
}

/// `%PRELAYHOME%` 展开为客户端主程序所在目录，由运行中的客户端进程解析。
pub(super) fn app_directory() -> Result<PathBuf, ClientError> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .ok_or_else(|| {
            ClientError::new("local_extensions_error", "无法确定 Prelay 客户端程序目录。")
        })
}

fn expand_user_directories(
    value: &str,
    home: &Path,
    app_directory: &Path,
) -> Result<String, ClientError> {
    let variables = [
        ("%USERPROFILE%".to_string(), home.display().to_string()),
        (
            "%APPDATA%".to_string(),
            home.join("AppData").join("Roaming").display().to_string(),
        ),
        (
            "%LOCALAPPDATA%".to_string(),
            home.join("AppData").join("Local").display().to_string(),
        ),
        (
            "%PRELAYHOME%".to_string(),
            app_directory.display().to_string(),
        ),
    ];
    let mut expanded = String::with_capacity(value.len());
    let mut rest = value;
    let mut replaced_variable = false;
    while let Some(index) = rest.find('%') {
        expanded.push_str(&rest[..index]);
        let tail = &rest[index..];
        if let Some(stripped) = tail.strip_prefix("%%") {
            expanded.push('%');
            rest = stripped;
            continue;
        }
        match variables
            .iter()
            .find(|(name, _)| tail.starts_with(name.as_str()))
        {
            Some((name, replacement)) => {
                expanded.push_str(replacement);
                replaced_variable = true;
                rest = &tail[name.len()..];
            }
            None => {
                if let Some(name) = unknown_variable_name(tail) {
                    return Err(ClientError::new(
                        "local_extensions_error",
                        format!("MCP 清单使用了未识别的目录变量 %{name}%"),
                    ));
                }
                expanded.push('%');
                rest = &tail[1..];
            }
        }
    }
    expanded.push_str(rest);
    if replaced_variable && MAIN_SEPARATOR != '\\' {
        expanded = expanded.replace('\\', MAIN_SEPARATOR_STR);
    }
    Ok(expanded)
}

/// 识别 `%NAME%` 形式的未知目录变量；普通文本中的 `%` 保持原样。
fn unknown_variable_name(tail: &str) -> Option<String> {
    let body = tail.strip_prefix('%')?;
    let end = body.find('%')?;
    let name = &body[..end];
    let mut characters = name.chars();
    if !characters.next()?.is_ascii_uppercase() {
        return None;
    }
    if !characters.all(|character| {
        character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
    }) {
        return None;
    }
    Some(name.to_string())
}

#[cfg(test)]
#[path = "mcp_manifest_tests.rs"]
mod tests;
