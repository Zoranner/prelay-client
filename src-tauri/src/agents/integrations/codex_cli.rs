use std::{
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::{ExtensionMcpManifest, ExtensionMcpTransport};
use toml_edit::{value, Array, DocumentMut, Item, Table, Value};

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::items::{remove_codex_config_item, write_text};
use crate::agents::{command_client_version, command_path, remove_skill_directory, scan_codex};

pub static CODEX_CLI: CodexCliIntegration = CodexCliIntegration;
pub struct CodexCliIntegration;

impl AgentIntegration for CodexCliIntegration {
    fn is_installed(&self) -> bool {
        command_path("codex").is_some()
    }
    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        scan_codex(home)
    }
    fn version(&self) -> Option<String> {
        command_path("codex").and_then(|path| command_client_version(&path))
    }

    fn rule_target(&self, home: &Path) -> Option<PathBuf> {
        Some(home.join(".codex").join("AGENTS.md"))
    }

    fn skill_target_root(&self, home: &Path) -> Option<PathBuf> {
        Some(home.join(".agents").join("skills"))
    }

    fn uninstall(
        &self,
        home: &Path,
        kind: AgentItemKind,
        name: &str,
        source_path: &str,
    ) -> Result<(), String> {
        match kind {
            AgentItemKind::Mcp => remove_codex_config_item(home, "mcp_servers", name),
            AgentItemKind::Skill => remove_skill_directory(source_path),
        }
    }
}

pub(crate) fn mcp_server_exists(home: &Path, name: &str) -> Result<bool, String> {
    let config_path = mcp_configuration_path(home);
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("无法读取 Codex 配置：{error}")),
    };
    let document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    Ok(document["mcp_servers"]
        .as_table()
        .is_some_and(|servers| servers.contains_key(name)))
}

pub(crate) fn mcp_server_matches(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<bool, String> {
    let config_path = mcp_configuration_path(home);
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("无法读取 Codex 配置：{error}")),
    };
    let document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    let Some(actual) = document["mcp_servers"]
        .as_table()
        .and_then(|servers| servers.get(&manifest.name))
        .and_then(Item::as_table)
    else {
        return Ok(false);
    };
    let expected = mcp_server_table(manifest)?;
    Ok(table_value(actual)? == table_value(&expected)?)
}

pub(crate) fn upsert_mcp_server(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<(), String> {
    let config_path = mcp_configuration_path(home);
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("无法读取 Codex 配置：{error}")),
    };
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex 配置不是有效的 TOML：{error}"))?;
    if !document.as_table().contains_key("mcp_servers") {
        document["mcp_servers"] = Item::Table(Table::new());
    }
    let servers = document["mcp_servers"]
        .as_table_mut()
        .ok_or_else(|| "Codex MCP 配置不是表。".to_string())?;
    servers.insert(&manifest.name, Item::Table(mcp_server_table(manifest)?));
    write_text(&config_path, document.to_string().as_bytes())
}

pub(crate) fn remove_mcp_server(home: &Path, name: &str) -> Result<(), String> {
    remove_codex_config_item(home, "mcp_servers", name)
}

fn mcp_configuration_path(home: &Path) -> PathBuf {
    home.join(".codex").join("config.toml")
}

fn mcp_server_table(manifest: &ExtensionMcpManifest) -> Result<Table, String> {
    let mut server = Table::new();
    match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            enabled,
            timeout_ms,
        } => {
            let (program, arguments) = command
                .split_first()
                .filter(|(program, _)| !program.trim().is_empty())
                .ok_or_else(|| "MCP 命令不能为空。".to_string())?;
            if cwd.is_some() {
                return Err("MCP 工作目录当前不受支持。".to_string());
            }
            if environment.iter().any(|(name, value)| name != value) {
                return Err("MCP 环境变量必须使用同名引用。".to_string());
            }
            server["command"] = value(program);
            set_string_array(&mut server, "args", arguments);
            set_string_array(
                &mut server,
                "env_vars",
                &environment.keys().cloned().collect::<Vec<_>>(),
            );
            server["enabled"] = value(*enabled);
            set_timeout(&mut server, *timeout_ms)?;
        }
        ExtensionMcpTransport::Http {
            url,
            headers,
            enabled,
            timeout_ms,
        } => {
            server["url"] = value(url);
            let mut header_variables = toml_edit::InlineTable::new();
            for (header, variable) in headers {
                header_variables.insert(header, Value::from(variable.as_str()));
            }
            if !header_variables.is_empty() {
                server["env_http_headers"] = Item::Value(Value::InlineTable(header_variables));
            }
            server["enabled"] = value(*enabled);
            set_timeout(&mut server, *timeout_ms)?;
        }
    }
    Ok(server)
}

fn table_value(table: &Table) -> Result<toml::Value, String> {
    toml::from_str(&table.to_string()).map_err(|error| format!("无法比较 Codex MCP 配置：{error}"))
}

fn set_string_array(table: &mut Table, key: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    let mut array = Array::new();
    for entry in values {
        array.push(entry.as_str());
    }
    table[key] = Item::Value(Value::Array(array));
}

fn set_timeout(table: &mut Table, timeout_ms: Option<u64>) -> Result<(), String> {
    let Some(timeout_ms) = timeout_ms else {
        return Ok(());
    };
    let timeout_secs = timeout_ms.div_ceil(1_000);
    let timeout_secs =
        i64::try_from(timeout_secs).map_err(|_| "MCP 超时时间超出 Codex 支持范围。".to_string())?;
    table["tool_timeout_sec"] = value(timeout_secs);
    Ok(())
}
