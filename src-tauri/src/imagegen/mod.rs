//! 通过 Prelay 接入点生成图像的 MCP stdio 服务端。
//!
//! 同时被 `imagegen` 可执行文件与测试使用，运行时不依赖 Tauri。

mod endpoint;
mod jsonrpc;
mod tools;

use std::io::{BufRead, Write};

use serde_json::{json, Value};

pub const SERVER_NAME: &str = "imagegen";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";

/// 按行处理 MCP 消息，直到标准输入关闭。
pub fn run_stdio() -> i32 {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some(response) = handle_line(trimmed) else {
            continue;
        };
        if writeln!(writer, "{response}").is_err() || writer.flush().is_err() {
            break;
        }
    }
    0
}

fn handle_line(line: &str) -> Option<String> {
    match jsonrpc::parse_request(line) {
        Ok(request) => handle_request(&request).map(|response| response.to_string()),
        Err(response) => Some(response.to_string()),
    }
}

fn handle_request(request: &jsonrpc::Request) -> Option<Value> {
    if request.method == "notifications/initialized" {
        return None;
    }
    let id = request.id.clone()?;
    let result = match request.method.as_str() {
        "initialize" => Ok(initialize_result(request)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools::list()),
        "tools/call" => Ok(match tools::call(&request.params) {
            Ok(text) => jsonrpc::tool_text(&text),
            Err(message) => jsonrpc::tool_error(&message),
        }),
        other => Err((jsonrpc::METHOD_NOT_FOUND, format!("未知方法：{other}"))),
    };
    Some(match result {
        Ok(result) => jsonrpc::success(&id, result),
        Err((code, message)) => jsonrpc::error(&id, code, &message),
    })
}

fn initialize_result(request: &jsonrpc::Request) -> Value {
    let protocol_version = request
        .params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_PROTOCOL_VERSION);
    json!({
        "protocolVersion": protocol_version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(line: &str) -> Value {
        serde_json::from_str(&handle_line(line).expect("line should produce a response")).unwrap()
    }

    #[test]
    fn answers_initialize_with_server_capabilities() {
        let response = response(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        );
        assert_eq!(response["result"]["protocolVersion"], json!("2025-06-18"));
        assert_eq!(response["result"]["serverInfo"]["name"], json!(SERVER_NAME));
        assert!(response["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn stays_quiet_on_notifications() {
        assert!(handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
    }

    #[test]
    fn reports_unknown_methods() {
        let response = response(r#"{"jsonrpc":"2.0","id":2,"method":"resources/list"}"#);
        assert_eq!(response["error"]["code"], json!(jsonrpc::METHOD_NOT_FOUND));
    }

    #[test]
    fn serves_tools_list() {
        let response = response(r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#);
        assert_eq!(
            response["result"]["tools"][0]["name"],
            json!("generate_image")
        );
    }

    #[test]
    fn reports_tool_failures_as_tool_errors() {
        let response = response(
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"generate_image","arguments":{"prompt":""}}}"#,
        );
        assert_eq!(response["result"]["isError"], json!(true));
    }
}
