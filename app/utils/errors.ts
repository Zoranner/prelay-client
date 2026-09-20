export interface RelayError {
  code: string;
  message: string;
}

/** 错误码到面向用户文案的唯一映射：管理 API 的稳定错误码 + 客户端本地错误码。 */
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
  invalid_credential: "设备凭据无效，需要重新注册。",
  not_found: "请求的对象不存在。",
  validation_failed: "请求内容不合法。",
  internal: "服务端发生内部错误。",
  credential_store_error: "本机设备凭据文件无法读取或写入。",
  relay_settings_error: "本机管理服务设置无法读取或写入。",
  relay_url_not_configured: "尚未配置管理服务地址。",
  desktop_preferences_error: "本机桌面偏好设置无法读取或写入。",
  local_agents_error: "读取本机智能体状态失败。",
  local_agent_settings_error: "读写本机智能体配置失败。",
  local_extensions_error: "本机扩展操作失败。",
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
  const code = Object.keys(errorMessages).find((knownCode) =>
    message.includes(knownCode),
  );
  return { code: code ?? "internal", message };
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

/**
 * 原始诊断信息。
 *
 * 已知 code 已经有确定文案，不再附带服务端或本地的原始消息；
 * 只有未识别的错误才把它作为兜底展示。
 */
export function errorDetail(error: RelayError): string | null {
  if (errorMessages[error.code]) {
    return null;
  }
  return error.message || null;
}

function httpStatusOf(message: string): string | undefined {
  return /^management API returned HTTP (\d{3})(?: .+)?$/.exec(message)?.[1];
}
