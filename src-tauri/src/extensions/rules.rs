use std::path::Path;

use prelay_protocol::ExtensionFile;

use crate::relay::client::ClientError;

use super::{atomic_write, decode_extension_file};

pub(super) fn install_rule(target: &Path, file: &ExtensionFile) -> Result<(), ClientError> {
    let content = decode_extension_file(file)?;
    std::str::from_utf8(&content)
        .map_err(|_| ClientError::new("invalid_response", "extension rule content is not UTF-8"))?;
    atomic_write(target, &content)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use prelay_protocol::ExtensionFile;
    use tempfile::tempdir;

    use super::install_rule;

    #[test]
    fn replaces_the_complete_rule_document() {
        let directory = tempdir().unwrap();
        let target = directory.path().join("AGENTS.md");
        fs::write(&target, "# Existing instructions\nKeep this").unwrap();

        install_rule(
            &target,
            &ExtensionFile {
                path: "AGENTS.md".to_string(),
                content_base64: BASE64.encode("# Published instructions"),
            },
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(target).unwrap(),
            "# Published instructions"
        );
    }

    #[test]
    fn rejects_rule_content_that_is_not_utf8() {
        let directory = tempdir().unwrap();
        let target = directory.path().join("AGENTS.md");

        let result = install_rule(
            &target,
            &ExtensionFile {
                path: "AGENTS.md".to_string(),
                content_base64: BASE64.encode([0xff]),
            },
        );

        assert!(result.is_err());
        assert!(!target.exists());
    }
}
