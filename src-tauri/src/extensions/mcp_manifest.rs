use prelay_protocol::{ExtensionFile, ExtensionMcpManifest, ExtensionMcpTransport};

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
    validate_mcp_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_mcp_manifest(manifest: &ExtensionMcpManifest) -> Result<(), ClientError> {
    if !is_mcp_server_name(&manifest.name) {
        return Err(ClientError::new("invalid_response", "MCP 服务名无效。"));
    }
    match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            enabled,
            ..
        } => {
            if command
                .first()
                .is_none_or(|program| program.trim().is_empty())
                || cwd.is_some()
                || !enabled
                || command_has_plaintext_secret(command)
                || environment.iter().any(|(name, value)| {
                    !is_environment_variable_name(name)
                        || !is_environment_variable_name(value)
                        || name != value
                })
            {
                return Err(ClientError::new(
                    "invalid_response",
                    "MCP 本地进程配置无效。",
                ));
            }
        }
        ExtensionMcpTransport::Http {
            url,
            headers,
            enabled,
            ..
        } => {
            if !enabled
                || !is_safe_http_url(url)
                || headers.iter().any(|(name, variable)| {
                    reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_err()
                        || !is_environment_variable_name(variable)
                })
            {
                return Err(ClientError::new("invalid_response", "MCP HTTP 配置无效。"));
            }
        }
    }
    Ok(())
}

fn is_mcp_server_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "workspace" | "cli" | "project" | "user" | "local" | "claude" | "builtin"
        )
}

fn is_safe_http_url(value: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https")
        && url.host().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && url.query().is_none()
}

fn command_has_plaintext_secret(command: &[String]) -> bool {
    command.iter().skip(1).any(|argument| {
        is_sensitive_command_option(argument) || contains_plaintext_credential(argument)
    })
}

fn is_sensitive_command_option(argument: &str) -> bool {
    let option = argument
        .trim_start_matches('-')
        .split_once('=')
        .map_or(argument.trim_start_matches('-'), |(name, _)| name);
    is_sensitive_name(option)
}

/// 位置参数里直接写成 URL 时，内嵌用户名密码或敏感 query 键同样属于明文凭据。
fn contains_plaintext_credential(argument: &str) -> bool {
    url_has_plaintext_credential(argument)
        || argument
            .split_once('=')
            .is_some_and(|(_, value)| url_has_plaintext_credential(value))
}

fn url_has_plaintext_credential(value: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    !url.username().is_empty()
        || url.password().is_some()
        || url.query_pairs().any(|(name, _)| is_sensitive_name(&name))
}

fn is_sensitive_name(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace('_', "-");
    matches!(
        normalized.as_str(),
        "api-key"
            | "apikey"
            | "access-key"
            | "accesskey"
            | "token"
            | "access-token"
            | "auth-token"
            | "refresh-token"
            | "secret"
            | "secret-key"
            | "client-secret"
            | "private-key"
            | "password"
            | "authorization"
            | "auth"
            | "bearer"
            | "credential"
            | "credentials"
            | "key"
            | "header"
            | "headers"
            | "signature"
            | "sig"
    )
}

fn is_environment_variable_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some('A'..='Z' | 'a'..='z' | '_'))
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[cfg(test)]
#[path = "mcp_manifest_tests.rs"]
mod tests;
