use std::{fs, io::Write, path::Path};

use atomic_write_file::AtomicWriteFile;
use toml_edit::{value, DocumentMut, Item, Table};

pub(super) fn read_toml(path: &Path) -> Option<toml::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok())
}

pub(super) fn read_json(path: &Path) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
}

pub(super) fn read_jsonc(path: &Path) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| json5::from_str(&contents).ok())
}

pub(super) fn read_optional_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

pub(super) fn read_toml_document(path: &Path) -> Result<DocumentMut, String> {
    match fs::read_to_string(path) {
        Ok(contents) => contents
            .parse()
            .map_err(|error| format!("Codex config is not valid TOML: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DocumentMut::new()),
        Err(error) => Err(format!("Codex config cannot be read: {error}")),
    }
}

pub(super) fn read_json_document(
    path: &Path,
    description: &str,
) -> Result<serde_json::Value, String> {
    match fs::read_to_string(path) {
        Ok(contents) if contents.trim().is_empty() => Ok(serde_json::json!({})),
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|error| format!("{description} is not valid JSON: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(error) => Err(format!("{description} cannot be read: {error}")),
    }
}

pub(super) fn read_jsonc_document(
    path: &Path,
    description: &str,
) -> Result<serde_json::Value, String> {
    match fs::read_to_string(path) {
        Ok(contents) if contents.trim().is_empty() => Ok(serde_json::json!({})),
        Ok(contents) => json5::from_str(&contents)
            .map_err(|error| format!("{description} is not valid JSONC: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(error) => Err(format!("{description} cannot be read: {error}")),
    }
}

pub(super) fn write_text(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("settings directory cannot be created: {error}"))?;
    let mut file = AtomicWriteFile::open(path)
        .map_err(|error| format!("settings file cannot be opened: {error}"))?;
    file.write_all(contents)
        .map_err(|error| format!("settings file cannot be written: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("settings file cannot be synchronized: {error}"))?;
    file.commit()
        .map_err(|error| format!("settings file cannot be committed: {error}"))
}

pub(super) fn table_mut<'a>(document: &'a mut DocumentMut, key: &str) -> &'a mut Table {
    if !document.as_table().contains_key(key) {
        document[key] = Item::Table(Table::new());
    }
    document[key]
        .as_table_mut()
        .expect("managed Codex settings section must be a table")
}

pub(super) fn set_item(document: &mut DocumentMut, key: &str, setting: Option<&str>) {
    match setting.filter(|value| !value.trim().is_empty()) {
        Some(setting) => document[key] = value(setting),
        None => {
            document.as_table_mut().remove(key);
        }
    }
}

pub(super) fn set_bool(document: &mut DocumentMut, key: &str, setting: Option<bool>) {
    match setting {
        Some(setting) => document[key] = value(setting),
        None => {
            document.as_table_mut().remove(key);
        }
    }
}

pub(super) fn set_table_string(table: &mut Table, key: &str, setting: Option<&str>) {
    match setting.filter(|value| !value.trim().is_empty()) {
        Some(setting) => table[key] = value(setting),
        None => {
            table.remove(key);
        }
    }
}

pub(super) fn set_table_bool(table: &mut Table, key: &str, setting: Option<bool>) {
    match setting {
        Some(setting) => table[key] = value(setting),
        None => {
            table.remove(key);
        }
    }
}

pub(super) fn set_table_integer(table: &mut Table, key: &str, setting: Option<u64>) {
    match setting {
        Some(setting) => table[key] = value(i64::try_from(setting).unwrap_or(i64::MAX)),
        None => {
            table.remove(key);
        }
    }
}

pub(super) fn toml_value<'a>(
    value: Option<&'a toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let mut current = value?;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

pub(super) fn toml_string(value: Option<&toml::Value>, path: &[&str]) -> Option<String> {
    toml_value(value, path)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}

pub(super) fn toml_bool(value: Option<&toml::Value>, path: &[&str]) -> Option<bool> {
    toml_value(value, path).and_then(toml::Value::as_bool)
}

pub(super) fn toml_integer(value: Option<&toml::Value>, path: &[&str]) -> Option<u64> {
    toml_value(value, path)
        .and_then(toml::Value::as_integer)
        .and_then(|number| u64::try_from(number).ok())
}

pub(super) fn toml_web_search(value: Option<&toml::Value>) -> Option<bool> {
    let setting = toml_value(value, &["web_search"])?;
    setting.as_bool().or_else(|| {
        setting
            .as_str()
            .map(|mode| !matches!(mode, "disabled" | "off" | "false"))
    })
}

pub(super) fn json_value<'a>(
    value: Option<&'a serde_json::Value>,
    path: &[&str],
) -> Option<&'a serde_json::Value> {
    let mut current = value?;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

pub(super) fn json_string(value: Option<&serde_json::Value>, path: &[&str]) -> Option<String> {
    json_value(value, path)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}
