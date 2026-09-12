use std::collections::BTreeMap;

use super::model::{ExtensionInstallAction, ExtensionPackage};
use prelay_protocol::{ExtensionKind, ExtensionMcpManifest, ExtensionMcpTransport};

pub(crate) const LOCAL_MCP_TEST_REPOSITORY: &str = "local://mcp-test";

pub fn local_mcp_test_package() -> ExtensionPackage {
    ExtensionPackage {
        name: "本地测试 MCP".to_string(),
        repository: LOCAL_MCP_TEST_REPOSITORY.to_string(),
        commit_sha: "local-mcp-test".to_string(),
        version: "v0.0.0-test".to_string(),
        kind: ExtensionKind::Mcp,
        install_action: ExtensionInstallAction::Install,
        installed_clients: Vec::new(),
    }
}

pub(crate) fn local_mcp_test_manifest() -> ExtensionMcpManifest {
    ExtensionMcpManifest {
        name: "prelay-mcp-test".to_string(),
        transport: ExtensionMcpTransport::Stdio {
            command: vec![
                "uvx".to_string(),
                "mcp-server-filesystem".to_string(),
                "%USERPROFILE%\\Documents".to_string(),
                "--mode".to_string(),
                "readonly".to_string(),
                "--watch".to_string(),
            ],
            cwd: None,
            environment: (1..=20)
                .map(|index| {
                    (
                        format!("MCP_TEST_API_KEY_{index:02}"),
                        format!("MCP_TEST_API_KEY_{index:02}"),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
            enabled: true,
            timeout_ms: Some(30_000),
        },
    }
}
