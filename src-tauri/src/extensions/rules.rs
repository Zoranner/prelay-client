use std::path::Path;

use prelay_protocol::ExtensionFile;

use crate::relay::client::ClientError;

use super::atomic_write;

pub(super) fn install_rule(target: &Path, file: &ExtensionFile) -> Result<(), ClientError> {
    atomic_write(target, file.content.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::fs;

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
                content: "# Published instructions".to_string(),
            },
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(target).unwrap(),
            "# Published instructions"
        );
    }
}
