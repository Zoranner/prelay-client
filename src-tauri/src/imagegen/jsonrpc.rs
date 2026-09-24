//! MCP stdio 传输使用的 JSON-RPC 2.0 消息解析与响应构造。

use serde_json::{json, Value};

pub(super) const PARSE_ERROR: i64 = -32700;
pub(super) const METHOD_NOT_FOUND: i64 = -32601;

#[derive(Debug, Clone)]
pub(super) struct Request {
    pub id: Option<Value>,
    pub method: String,
    pub params: Value,
}

/// 解析一行 MCP 消息；解析失败时返回可以直接回写的错误响应。
pub(super) fn parse_request(line: &str) -> Result<Request, Value> {
    let value: Value = serde_json::from_str(line)
        .map_err(|_| error(&Value::Null, PARSE_ERROR, "无法解析 JSON-RPC 消息"))?;
    let method = value
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| error(&Value::Null, PARSE_ERROR, "JSON-RPC 消息缺少 method"))?;
    Ok(Request {
        id: value.get("id").cloned().filter(|id| !id.is_null()),
        method: method.to_string(),
        params: value.get("params").cloned().unwrap_or_else(|| json!({})),
    })
}

pub(super) fn success(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

pub(super) fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// 工具执行结果：成功与失败都走 `content`，失败时置 `isError`。
pub(super) fn tool_text(text: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

pub(super) fn tool_error(message: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": message }], "isError": true })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_requests_with_and_without_ids() {
        let request = parse_request(r#"{"jsonrpc":"2.0","id":7,"method":"tools/list"}"#).unwrap();
        assert_eq!(request.id, Some(json!(7)));
        assert_eq!(request.method, "tools/list");
        assert_eq!(request.params, json!({}));

        let notification =
            parse_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).unwrap();
        assert_eq!(notification.id, None);
    }

    #[test]
    fn reports_parse_failures_as_json_rpc_errors() {
        let error = parse_request("not-json").unwrap_err();
        assert_eq!(error["error"]["code"], json!(PARSE_ERROR));
        assert_eq!(error["id"], Value::Null);
    }

    #[test]
    fn builds_tool_results() {
        assert_eq!(
            tool_text("done")["content"][0]["text"],
            Value::String("done".to_string())
        );
        assert_eq!(tool_error("boom")["isError"], Value::Bool(true));
    }
}
