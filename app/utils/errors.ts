export interface RelayError {
  code: string;
  message: string;
}

/**
 * 错误码到面向用户文案的唯一映射。
 *
 * 前半部分是 `ProtocolErrorCode` 的稳定码；后半部分是客户端本地码，
 * 以及服务端未返回可解析信封时按 HTTP 状态生成的兜底码。
 */
const errorMessages: Record<string, string> = {
  identity_already_registered: "该 Windows 身份已在服务端注册，无法自动恢复。",
  client_update_unavailable: "服务端当前没有可用的客户端更新包。",
  extension_catalog_unavailable: "扩展目录暂时不可用。",
  extension_content_invalid: "扩展内容不合法。",
  extension_not_found: "没有找到该扩展。",
  extension_version_not_found: "没有找到该扩展版本。",
  extension_install_unsupported: "该类型的扩展不支持本机安装。",
  invalid_provider_sharing: "供应商共享配置不合法。",
  provider_sharing_not_allowed: "只有供应商所有者可以管理共享。",
  provider_not_visible: "该供应商对当前身份不可见。",
  provider_not_usable: "该供应商当前不可用。",
  provider_route_unavailable: "该接入点没有可用的上游路由。",
  provider_upstream_unavailable: "无法连接该供应商的上游服务。",
  provider_model_retired: "供应商已下架部分模型，请先移除后再保存。",
  provider_type_unsupported: "该供应商类型已不可用，无法新建或编辑。",
  endpoint_model_duplicate:
    "同一个供应商下存在重复的模型绑定，请先移除重复项。",
  endpoint_model_not_in_provider_models:
    "所选模型不在供应商已启用的模型清单里，请先移除或改选。",
  invalid_credential: "设备凭据无效，需要重新注册。",
  not_found: "请求的对象不存在。",
  validation_failed: "请求内容不合法。",
  internal: "服务端发生内部错误。",
  forbidden: "当前身份没有执行该操作的权限。",
  rate_limited: "请求过于频繁，请稍后重试。",
  server_error: "管理服务暂时不可用。",
  credential_store_error: "本机设备凭据文件无法读取或写入。",
  relay_settings_error: "本机管理服务设置无法读取或写入。",
  relay_url_not_configured: "尚未配置管理服务地址。",
  desktop_preferences_error: "本机桌面偏好设置无法读取或写入。",
  local_agents_error: "读取本机智能体状态失败。",
  local_agent_settings_error: "读写本机智能体配置失败。",
  local_extensions_error: "本机扩展操作失败。",
  local_extensions_conflict: "本机扩展状态存在归属冲突，请先处理重复声明。",
  local_extensions_readonly: "本机扩展状态版本过高，客户端已进入只读模式。",
  invalid_response: "服务端返回的内容无法识别。",
  invalid_request: "本地请求构造失败。",
  missing_device_credential: "本机没有可用的设备凭据，需要重新注册。",
  client_update_storage_error: "更新包在本地写入失败。",
  client_update_install_failed: "启动安装程序失败。",
  client_update_not_downloaded: "更新包尚未下载完成。",
  client_update_unsupported_platform: "客户端更新只支持 Windows。",
  invalid_client_update: "客户端更新包不合法。",
  invalid_relay_url: "服务地址必须是 HTTP 或 HTTPS 链接。",
  extension_target_exists: "目标位置已存在同名内容，需要确认覆盖。",
  network_error: "无法连接到管理服务。",
  upstream_connect_timeout: "上游连接超时。",
  upstream_connect_failed: "无法连接上游服务。",
  upstream_test_failed: "上游测试失败。",
  upstream_status: "上游返回了错误状态。",
};

/**
 * 活动记录的稳定错误码到文案。
 *
 * 活动记录的错误码是独立码空间，只在活动详情里展示；
 * 原始 `error_message` 作为诊断信息单独呈现。
 */
const activityErrorMessages: Record<string, string> = {
  upstream_status: "上游服务返回了错误状态。",
  upstream_connect: "无法连接上游服务。",
  upstream_response_head_timeout: "上游在规定时间内没有返回响应。",
  upstream_invalid_response: "上游返回的内容无法识别。",
  upstream_body: "读取上游响应失败。",
  upstream_error: "上游请求失败。",
  empty_stream: "上游流式响应没有返回内容。",
  stream_error: "流式响应中断。",
  downstream_disconnect: "客户端在响应完成前断开。",
  activity_rollout_incomplete: "活动记录未完整写入。",
  activity_interrupted: "服务重启导致活动记录中断。",
  shared_quota_exhausted: "共享额度不足。",
};

const httpStatusExplanations: Record<string, string> = {
  "400": "拒绝了请求内容",
  "401": "拒绝了当前设备凭据",
  "403": "拒绝了当前身份的访问",
  "404": "未找到请求的管理接口",
  "405": "不支持本次请求使用的方法",
  "500": "发生内部错误",
};

export function toRelayError(error: unknown): RelayError {
  if (typeof error === "object" && error !== null) {
    const candidate = error as Partial<RelayError>;
    if (
      typeof candidate.code === "string" &&
      typeof candidate.message === "string"
    ) {
      return { code: candidate.code, message: candidate.message };
    }
  }

  const message = error instanceof Error ? error.message : String(error);
  return { code: legacyCodeFromMessage(message) ?? "unknown", message };
}

/**
 * 兼容旧版 Tauri 的 `code: message` 字符串。
 *
 * 只接受行首的已知码，不在整段消息里搜索子串；未识别的码留给兜底展示。
 */
function legacyCodeFromMessage(message: string): string | undefined {
  const match = /^([a-z][a-z0-9_]*): /.exec(message);
  if (!match) {
    return undefined;
  }
  const code = match[1];
  return code && errorMessages[code] ? code : undefined;
}

/** 面向用户的错误文案：已知 code 固定中文，未识别的 code 回退到原始消息。 */
export function errorText(error: RelayError): string {
  const known = errorMessages[error.code];
  if (known) {
    return known;
  }

  const statusCode = httpStatusOf(error.message);
  if (statusCode) {
    const explanation = httpStatusExplanations[statusCode] ?? "拒绝了本次请求";
    return `管理服务${explanation}（HTTP ${statusCode}）。`;
  }

  return error.message || "发生了未知错误。";
}

/** 活动记录错误码的用户文案；未登记的码返回 null，由界面回退到原始码。 */
export function activityErrorText(code: string | null): string | null {
  if (!code) {
    return null;
  }
  return activityErrorMessages[code] ?? null;
}

function httpStatusOf(message: string): string | undefined {
  return /^management API returned HTTP (\d{3})(?: .+)?$/.exec(message)?.[1];
}
