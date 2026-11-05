use std::{fs, path::Path};

use prelay_protocol::ExtensionFile;

use crate::relay::client::ClientError;

use super::{
    atomic_write, decode_extension_file,
    host_state::{hash_bytes, ArtifactHealth, RuleEntry, RuleState},
};

pub(crate) fn plan_rule(
    package: &str,
    version: &str,
    commit_sha: &str,
    file: &ExtensionFile,
) -> Result<(Vec<u8>, RuleEntry), ClientError> {
    let content = decode_extension_file(file)?;
    std::str::from_utf8(&content)
        .map_err(|_| ClientError::new("invalid_response", "extension rule content is not UTF-8"))?;
    Ok((
        content.clone(),
        RuleEntry {
            package: package.to_string(),
            version: version.to_string(),
            commit_sha: commit_sha.to_string(),
            content_hash: hash_bytes(&content),
            state: super::host_state::PluginState::Installing,
        },
    ))
}

pub(crate) fn write_rule(target: &Path, content: &[u8]) -> Result<(), ClientError> {
    atomic_write(target, content)
}

pub(crate) fn health(target: &Path, entry: &RuleEntry) -> ArtifactHealth {
    match fs::read(target) {
        Ok(contents) if hash_bytes(&contents) == entry.content_hash => ArtifactHealth::Managed,
        Ok(_) => ArtifactHealth::Modified,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ArtifactHealth::Missing,
        Err(_) => ArtifactHealth::Missing,
    }
}

pub(crate) fn remember(state: &mut RuleState, target: &Path, entry: RuleEntry) {
    state
        .targets
        .insert(super::host_state::normalize_path(target), entry);
}

pub(crate) fn forget(state: &mut RuleState, target: &Path) -> Option<RuleEntry> {
    state
        .targets
        .remove(&super::host_state::normalize_path(target))
}
