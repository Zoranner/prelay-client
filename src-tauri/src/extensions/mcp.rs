use std::path::Path;

use prelay_protocol::{ExtensionFile, ExtensionMcpManifest, ExtensionMcpTransport};

use crate::{
    agents::{upsert_codex_mcp_server, AgentClient},
    relay::client::ClientError,
};

const MCP_MANIFEST_PATH: &str = "server.json";

pub(super) fn read_manifest(files: &[ExtensionFile]) -> Result<ExtensionMcpManifest, ClientError> {
    let file = files
        .iter()
        .find(|file| file.path == MCP_MANIFEST_PATH)
        .ok_or_else(|| ClientError::new("invalid_response", "MCP 安装包缺少 server.json。"))?;
    let manifest = serde_json::from_str::<ExtensionMcpManifest>(&file.content)
        .map_err(|_| ClientError::new("invalid_response", "MCP server.json 格式无效。"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub(super) fn install_mcp(
    home: &Path,
    clients: &[AgentClient],
    manifest: &ExtensionMcpManifest,
) -> Result<(), ClientError> {
    if clients
        .iter()
        .any(|client| matches!(client, AgentClient::CodexCli | AgentClient::ChatGpt))
    {
        upsert_codex_mcp_server(home, manifest).map_err(local_error)?;
    }
    if clients
        .iter()
        .any(|client| matches!(client, AgentClient::OpenCode))
    {
        crate::agents::integrations::opencode::upsert_mcp_server(home, manifest)
            .map_err(local_error)?;
    }
    Ok(())
}

fn validate_manifest(manifest: &ExtensionMcpManifest) -> Result<(), ClientError> {
    if manifest.name.trim().is_empty() {
        return Err(ClientError::new("invalid_response", "MCP 名称不能为空。"));
    }
    match &manifest.transport {
        ExtensionMcpTransport::Stdio { command, .. }
            if !command.is_empty() && command.iter().all(|entry| !entry.is_empty()) =>
        {
            Ok(())
        }
        ExtensionMcpTransport::Http { url, .. }
            if url.starts_with("https://") || url.starts_with("http://") =>
        {
            Ok(())
        }
        _ => Err(ClientError::new("invalid_response", "MCP 传输配置无效。")),
    }
}

fn local_error(message: String) -> ClientError {
    ClientError::new("local_extensions_error", message)
}

#[cfg(test)]
mod tests {
    use prelay_protocol::ExtensionFile;

    use super::read_manifest;

    #[test]
    fn rejects_an_mcp_manifest_without_a_command() {
        let result = read_manifest(&[ExtensionFile {
            path: "server.json".to_string(),
            content: r#"{
                "name": "research",
                "transport": {
                    "type": "stdio",
                    "command": [],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#
            .to_string(),
        }]);

        assert!(result.is_err());
    }
}
