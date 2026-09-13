use prelay_protocol::{
    validate_mcp_manifest as validate_manifest, ExtensionFile, ExtensionMcpManifest,
    McpManifestError,
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

#[cfg(test)]
#[path = "mcp_manifest_tests.rs"]
mod tests;
