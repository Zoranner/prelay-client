use std::fs;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::ExtensionFile;
use serde_json::Value;

pub(super) fn hook_file(value: Value) -> ExtensionFile {
    ExtensionFile {
        path: "hooks/hooks.json".to_string(),
        content_base64: BASE64.encode(serde_json::to_vec(&value).unwrap()),
    }
}

pub(super) fn support_file(path: &str, content: &str) -> ExtensionFile {
    ExtensionFile {
        path: path.to_string(),
        content_base64: BASE64.encode(content),
    }
}

pub(super) fn binary_support_file(path: &str, content: &[u8]) -> ExtensionFile {
    ExtensionFile {
        path: path.to_string(),
        content_base64: BASE64.encode(content),
    }
}

pub(super) fn hooks_path(home: &std::path::Path) -> std::path::PathBuf {
    home.join(".codex").join("hooks.json")
}

pub(super) fn hook_root(home: &std::path::Path, commit: &str) -> std::path::PathBuf {
    home.join(".prelay")
        .join("packages")
        .join("superpowers")
        .join(commit)
        .join("hooks")
}

pub(super) fn hooks_document(home: &std::path::Path) -> Value {
    serde_json::from_slice(
        &fs::read(home.join(".codex").join("hooks.json")).expect("read hooks.json"),
    )
    .expect("parse hooks.json")
}

pub(super) fn commands(home: &std::path::Path) -> Vec<String> {
    hooks_document(home)["hooks"]["SessionStart"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .filter_map(|handler| handler.get("command").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}
