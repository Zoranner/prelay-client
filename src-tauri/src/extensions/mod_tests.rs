use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::{ExtensionFile, ExtensionInstallBundle, ExtensionKind, ExtensionVersion};

use super::{validate_package_bundle, ExtensionInstallAction, ExtensionPackage};

fn mcp_package() -> ExtensionPackage {
    ExtensionPackage {
        name: "filesystem-mcp".to_string(),
        repository: "https://git.example.test/agents/filesystem-mcp".to_string(),
        commit_sha: "expected-commit".to_string(),
        version: "v1.0.0".to_string(),
        kind: ExtensionKind::Mcp,
        install_action: ExtensionInstallAction::Install,
        installed_clients: Vec::new(),
    }
}

fn mcp_bundle(commit_sha: &str) -> ExtensionInstallBundle {
    ExtensionInstallBundle {
        name: "filesystem-mcp".to_string(),
        kind: ExtensionKind::Mcp,
        version: ExtensionVersion {
            tag: "v1.0.0".to_string(),
            commit_sha: commit_sha.to_string(),
            updated_at: "2026-09-11T00:00:00Z".to_string(),
        },
        files: vec![ExtensionFile {
            path: "server.json".to_string(),
            content_base64: BASE64.encode(
                r#"{
                    "name": "filesystem",
                    "transport": {
                        "type": "stdio",
                        "command": ["uvx", "mcp-server-filesystem"],
                        "cwd": null,
                        "environment": {},
                        "enabled": true,
                        "timeoutMs": null
                    }
                }"#,
            ),
        }],
    }
}

#[test]
fn rejects_an_install_bundle_that_does_not_match_the_listed_commit() {
    let package = mcp_package();

    assert!(validate_package_bundle(&package, &mcp_bundle("expected-commit")).is_ok());
    assert!(validate_package_bundle(&package, &mcp_bundle("moved-tag-commit")).is_err());
}
