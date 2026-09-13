use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::ExtensionFile;

use super::read_mcp_manifest;

#[test]
fn rejects_unsafe_mcp_bundles() {
    for content in [
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem"],
                    "cwd": null,
                    "environment": { "GITHUB_TOKEN": "plain-text-secret" },
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem"],
                    "cwd": null,
                    "environment": {},
                    "enabled": false,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem server",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "workspace",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem", "--api-key", "plain-text-secret"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "remote",
                "transport": {
                    "type": "http",
                    "url": "https://mcp.example.test?token=plain-text-secret",
                    "headers": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem", "--access-key", "plain-text-secret"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "remote",
                "transport": {
                    "type": "http",
                    "url": "https://mcp.example.test?transport=stream",
                    "headers": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "remote",
                "transport": {
                    "type": "http",
                    "url": "https://mcp.example.test?api_key=plain-text-secret",
                    "headers": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-remote", "--header", "Authorization: Bearer plain-text-secret"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-remote", "https://user:plain-text-secret@mcp.example.test/mcp"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-remote", "https://mcp.example.test/mcp?access_token=plain-text-secret"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem", "--client-secret=plain-text-secret"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
    ] {
        let file = ExtensionFile {
            path: "server.json".to_string(),
            content_base64: BASE64.encode(content),
        };

        assert!(read_mcp_manifest(&file).is_err());
    }
}

#[test]
fn accepts_ordinary_mcp_command_arguments() {
    for content in [
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-server-filesystem", "--tokenizer=cl100k_base"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
        r#"{
                "name": "filesystem",
                "transport": {
                    "type": "stdio",
                    "command": ["uvx", "mcp-remote", "https://docs.example.test/guide"],
                    "cwd": null,
                    "environment": {},
                    "enabled": true,
                    "timeoutMs": null
                }
            }"#,
    ] {
        let file = ExtensionFile {
            path: "server.json".to_string(),
            content_base64: BASE64.encode(content),
        };

        assert!(read_mcp_manifest(&file).is_ok());
    }
}
