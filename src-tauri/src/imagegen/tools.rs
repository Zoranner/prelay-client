//! MCP 工具定义与调用实现。

use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::Engine as _;
use serde_json::{json, Value};

use super::endpoint;

const DEFAULT_MODEL: &str = "gpt-image-2";
const DEFAULT_OUTPUT_DIRECTORY: &str = "output/imagegen";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);
const ERROR_BODY_LIMIT: usize = 300;

pub(super) fn list() -> Value {
    json!({ "tools": [generate_image_tool()] })
}

fn generate_image_tool() -> Value {
    json!({
        "name": "generate_image",
        "description": "通过本机 Prelay 接入点生成图像并把结果保存到工作区文件。需要生成或产出图片时使用本工具。",
        "inputSchema": {
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "图像生成提示词" },
                "model": { "type": "string", "description": "图像模型 id，默认 gpt-image-2" },
                "size": { "type": "string", "description": "图像尺寸，例如 1024x1024" },
                "quality": { "type": "string", "enum": ["low", "medium", "high", "auto"] },
                "output_format": { "type": "string", "enum": ["png", "jpeg", "webp"] },
                "out": { "type": "string", "description": "输出文件相对路径，默认 output/imagegen/imagegen-<时间戳>.<格式>" }
            },
            "required": ["prompt"],
            "additionalProperties": false
        }
    })
}

pub(super) fn call(params: &Value) -> Result<String, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "缺少工具名称".to_string())?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match name {
        "generate_image" => generate_image(&arguments),
        other => Err(format!("未知工具：{other}")),
    }
}

fn generate_image(arguments: &Value) -> Result<String, String> {
    let prompt =
        string_argument(arguments, "prompt").ok_or_else(|| "prompt 不能为空".to_string())?;
    let model = string_argument(arguments, "model").unwrap_or(DEFAULT_MODEL);
    let output_format = string_argument(arguments, "output_format").unwrap_or("png");
    let mut payload = json!({ "model": model, "prompt": prompt });
    for name in ["size", "quality"] {
        if let Some(value) = string_argument(arguments, name) {
            payload[name] = json!(value);
        }
    }
    if let Some(value) = string_argument(arguments, "output_format") {
        payload["output_format"] = json!(value);
    }

    let endpoint = endpoint::resolve()?;
    let response = post_generation(&endpoint, &payload)?;
    let image = first_image_bytes(&response)?;
    let path = output_path(arguments, output_format)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建输出目录（{}）：{error}", parent.display()))?;
    }
    fs::write(&path, &image)
        .map_err(|error| format!("无法写入图像文件（{}）：{error}", path.display()))?;
    Ok(format!(
        "已生成图像：{}（模型 {model}，{} 字节）",
        path.display(),
        image.len()
    ))
}

fn post_generation(endpoint: &endpoint::Endpoint, payload: &Value) -> Result<Value, String> {
    let url = format!("{}/images/generations", endpoint.base_url);
    let client = reqwest::blocking::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| format!("无法创建 HTTP 客户端：{error}"))?;
    let response = client
        .post(&url)
        .bearer_auth(&endpoint.token)
        .json(payload)
        .send()
        .map_err(|error| format!("无法连接 Prelay 接入点：{error}"))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| format!("读取接入点响应失败：{error}"))?;
    if !status.is_success() {
        return Err(format!(
            "接入点返回 {status}：{}",
            truncate(&body, ERROR_BODY_LIMIT)
        ));
    }
    serde_json::from_str(&body).map_err(|error| format!("接入点响应不是有效 JSON：{error}"))
}

fn first_image_bytes(response: &Value) -> Result<Vec<u8>, String> {
    let data = response
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "接入点响应缺少 data 数组".to_string())?;
    let first = data
        .first()
        .ok_or_else(|| "接入点响应没有返回图像".to_string())?;
    let encoded = first
        .get("b64_json")
        .and_then(Value::as_str)
        .ok_or_else(|| "接入点响应缺少 b64_json，当前只支持 base64 图像".to_string())?;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| format!("图像数据无法解码：{error}"))
}

fn output_path(arguments: &Value, output_format: &str) -> Result<PathBuf, String> {
    let current = std::env::current_dir().map_err(|error| format!("无法确定当前目录：{error}"))?;
    match string_argument(arguments, "out") {
        Some(relative) => {
            let candidate = Path::new(relative);
            if candidate.is_absolute()
                || candidate.components().any(|component| {
                    matches!(
                        component,
                        Component::ParentDir | Component::RootDir | Component::Prefix(_)
                    )
                })
            {
                return Err("out 必须是工作区内不含 .. 的相对路径".to_string());
            }
            Ok(current.join(candidate))
        }
        None => Ok(current.join(DEFAULT_OUTPUT_DIRECTORY).join(format!(
            "imagegen-{}.{}",
            timestamp(),
            file_extension(output_format)
        ))),
    }
}

fn file_extension(output_format: &str) -> &'static str {
    match output_format {
        "jpeg" | "jpg" => "jpg",
        "webp" => "webp",
        _ => "png",
    }
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn string_argument<'a>(arguments: &'a Value, name: &str) -> Option<&'a str> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_the_generate_image_tool() {
        let tools = list();
        assert_eq!(tools["tools"][0]["name"], json!("generate_image"));
        assert_eq!(
            tools["tools"][0]["inputSchema"]["required"][0],
            json!("prompt")
        );
    }

    #[test]
    fn rejects_unknown_tools_and_empty_prompts() {
        let error = call(&json!({ "name": "embed_text", "arguments": {} })).unwrap_err();
        assert!(error.contains("embed_text"));

        let error = call(&json!({ "name": "generate_image", "arguments": { "prompt": "  " } }))
            .unwrap_err();
        assert!(error.contains("prompt"));
    }

    #[test]
    fn decodes_base64_images() {
        let response = json!({ "data": [{ "b64_json": "aGVsbG8=" }] });
        assert_eq!(first_image_bytes(&response).unwrap(), b"hello");

        let error = first_image_bytes(&json!({ "data": [] })).unwrap_err();
        assert!(error.contains("没有返回图像"));
    }

    #[test]
    fn keeps_output_paths_inside_the_workspace() {
        let error = output_path(&json!({ "out": "..\\escape.png" }), "png").unwrap_err();
        assert!(error.contains("相对路径"));

        let path = output_path(&json!({ "out": "output/imagegen/puppy.png" }), "png").unwrap();
        assert!(path.ends_with(Path::new("output").join("imagegen").join("puppy.png")));

        let default = output_path(&json!({}), "jpeg").unwrap();
        assert_eq!(
            default.extension().and_then(|value| value.to_str()),
            Some("jpg")
        );
        assert!(default.to_string_lossy().contains("output"));
    }

    #[test]
    fn truncates_long_error_bodies() {
        let body = "x".repeat(400);
        let truncated = truncate(&body, ERROR_BODY_LIMIT);
        assert_eq!(truncated.chars().count(), ERROR_BODY_LIMIT + 1);
        assert!(truncated.ends_with('…'));
    }
}
