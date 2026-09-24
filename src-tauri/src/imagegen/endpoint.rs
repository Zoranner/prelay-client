//! 从本机 Codex 配置解析当前 Prelay 接入点。

use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Endpoint {
    pub base_url: String,
    pub token: String,
}

/// 读取 `~/.codex/config.toml` 与 `~/.codex/auth.json`。
pub(super) fn resolve() -> Result<Endpoint, String> {
    let home = env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| "无法确定用户目录（USERPROFILE 未设置）".to_string())?;
    let codex_home = home.join(".codex");
    let config = read_file(&codex_home.join("config.toml"), "Codex config.toml")?;
    let auth = read_file(&codex_home.join("auth.json"), "Codex auth.json")?;
    resolve_from(&config, &auth)
}

pub(super) fn resolve_from(config_toml: &str, auth_json: &str) -> Result<Endpoint, String> {
    let config: toml::Value = toml::from_str(config_toml)
        .map_err(|error| format!("Codex config.toml 无法解析：{error}"))?;
    let provider_id = config
        .get("model_provider")
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("custom");
    let base_url = config
        .get("model_providers")
        .and_then(|providers| providers.get(provider_id))
        .and_then(|provider| provider.get("base_url"))
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Codex config.toml 缺少 model_providers.{provider_id}.base_url"))?;
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err("Codex 接入点 base_url 必须是 http 或 https 地址".to_string());
    }
    let auth: serde_json::Value = serde_json::from_str(auth_json)
        .map_err(|error| format!("Codex auth.json 无法解析：{error}"))?;
    let token = auth
        .get("OPENAI_API_KEY")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Codex auth.json 缺少 OPENAI_API_KEY".to_string())?;
    Ok(Endpoint {
        base_url: base_url.trim_end_matches('/').to_string(),
        token: token.to_string(),
    })
}

fn read_file(path: &Path, label: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("无法读取 {label}（{}）：{error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_active_provider_endpoint() {
        let config = r#"
model_provider = "custom"

[model_providers.custom]
base_url = "https://prelay.example/v1/"
"#;
        let endpoint = resolve_from(config, r#"{"OPENAI_API_KEY":"token-value"}"#).unwrap();
        assert_eq!(endpoint.base_url, "https://prelay.example/v1");
        assert_eq!(endpoint.token, "token-value");
    }

    #[test]
    fn falls_back_to_the_custom_provider() {
        let config = r#"
[model_providers.custom]
base_url = "https://prelay.example/v1"
"#;
        let endpoint = resolve_from(config, r#"{"OPENAI_API_KEY":"token-value"}"#).unwrap();
        assert_eq!(endpoint.base_url, "https://prelay.example/v1");
    }

    #[test]
    fn rejects_incomplete_configuration() {
        let error = resolve_from("model_provider = \"custom\"", "{}").unwrap_err();
        assert!(error.contains("base_url"));

        let error = resolve_from(
            "[model_providers.custom]\nbase_url = \"https://prelay.example/v1\"\n",
            "{}",
        )
        .unwrap_err();
        assert!(error.contains("OPENAI_API_KEY"));
    }
}
