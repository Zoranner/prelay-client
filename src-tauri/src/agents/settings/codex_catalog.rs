use prelay_protocol::CatalogLanguageModelResponse;
use serde::Serialize;
use serde_json::{json, Map, Value};

pub(super) fn codex_model_profile(model: &CatalogLanguageModelResponse) -> Result<Value, String> {
    let mut profile = Map::new();
    let reasoning_efforts = model.reasoning_efforts.as_deref().unwrap_or_default();
    profile.insert("slug".to_string(), Value::String(model.id.clone()));
    profile.insert(
        "display_name".to_string(),
        Value::String(model.display_name.clone()),
    );
    profile.insert(
        "supported_reasoning_levels".to_string(),
        Value::Array(
            reasoning_efforts
                .iter()
                .map(|effort| {
                    json!({
                        "effort": effort,
                        "description": reasoning_effort_description(effort),
                    })
                })
                .collect(),
        ),
    );

    insert_optional(&mut profile, "description", model.description.as_ref())?;
    if let Some(default_effort) =
        model
            .default_reasoning_effort
            .as_deref()
            .filter(|default_effort| {
                reasoning_efforts
                    .iter()
                    .any(|effort| effort == default_effort)
            })
    {
        profile.insert(
            "default_reasoning_level".to_string(),
            Value::String(default_effort.to_string()),
        );
    }
    profile.insert(
        "shell_type".to_string(),
        Value::String(
            model
                .shell_type
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("shell_command")
                .to_string(),
        ),
    );
    profile.insert(
        "visibility".to_string(),
        Value::String(
            model
                .visibility
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("list")
                .to_string(),
        ),
    );
    profile.insert(
        "supported_in_api".to_string(),
        Value::Bool(model.supported_in_api.unwrap_or(true)),
    );
    profile.insert(
        "priority".to_string(),
        Value::Number(model.priority.unwrap_or(0).into()),
    );
    profile.insert(
        "base_instructions".to_string(),
        Value::String(model.base_instructions.clone().unwrap_or_default()),
    );
    insert_optional(
        &mut profile,
        "context_window",
        model.context_window.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "max_context_window",
        model.max_context_window.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "effective_context_window_percent",
        model.effective_context_window_percent.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "input_modalities",
        model.input_modalities.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "supports_parallel_tool_calls",
        model.supports_parallel_tool_calls.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "supports_reasoning_summary_parameter",
        model.supports_reasoning_summaries.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "supports_image_detail_original",
        model.supports_image_detail_original.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "support_verbosity",
        model.support_verbosity.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "default_verbosity",
        model.default_verbosity.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "apply_patch_tool_type",
        model.apply_patch_tool_type.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "web_search_tool_type",
        model.web_search_tool_type.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "truncation_policy",
        model.truncation_policy.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "reasoning_summary_format",
        model.reasoning_summary_format.as_ref(),
    )?;
    insert_optional(
        &mut profile,
        "default_reasoning_summary",
        model.default_reasoning_summary.as_ref(),
    )?;
    profile.insert(
        "experimental_supported_tools".to_string(),
        serde_json::to_value(
            model
                .experimental_supported_tools
                .as_deref()
                .unwrap_or_default(),
        )
        .map_err(|error| {
            format!("Codex 模型档案字段 experimental_supported_tools 无法序列化: {error}")
        })?,
    );
    insert_optional(
        &mut profile,
        "minimal_client_version",
        model.minimal_client_version.as_ref(),
    )?;
    if model
        .web_search_tool_type
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        profile.insert("supports_search_tool".to_string(), Value::Bool(true));
    }
    Ok(Value::Object(profile))
}

fn insert_optional<T: Serialize>(
    profile: &mut Map<String, Value>,
    key: &str,
    value: Option<&T>,
) -> Result<(), String> {
    if let Some(value) = value {
        let value = serde_json::to_value(value)
            .map_err(|error| format!("Codex 模型档案字段 {key} 无法序列化: {error}"))?;
        profile.insert(key.to_string(), value);
    }
    Ok(())
}

fn reasoning_effort_description(effort: &str) -> &'static str {
    match effort {
        "none" => "No reasoning",
        "minimal" => "Minimal reasoning depth",
        "low" => "Fast responses with lighter reasoning",
        "medium" => "Balanced reasoning depth",
        "high" => "Extra reasoning depth for complex problems",
        "xhigh" => "Very high reasoning depth for complex problems",
        "max" => "Maximum reasoning depth for the hardest problems",
        _ => "Model-defined reasoning depth",
    }
}
