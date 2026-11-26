import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import {
  activityErrorText,
  errorText,
  toRelayError,
} from "../app/utils/errors";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("管理 API 错误码使用中文文案", () => {
  for (const code of [
    "internal",
    "invalid_credential",
    "not_found",
    "validation_failed",
    "identity_already_registered",
    "provider_not_visible",
    "provider_not_usable",
    "provider_sharing_not_allowed",
    "provider_upstream_unavailable",
    "provider_model_retired",
    "provider_type_unsupported",
    "endpoint_model_duplicate",
    "endpoint_model_not_in_provider_models",
    "extension_catalog_unavailable",
    "client_update_unavailable",
  ]) {
    const text = errorText({ code, message: "server diagnostic" });

    expect(text).not.toBe("server diagnostic");
    expect(text).not.toBe("");
    expect(text).toMatch(/[\u4e00-\u9fa5]/);
  }
});

test("客户端本地错误码使用中文文案", () => {
  for (const code of [
    "network_error",
    "credential_store_error",
    "relay_settings_error",
    "relay_url_not_configured",
    "invalid_relay_url",
    "desktop_preferences_error",
    "local_agents_error",
    "local_agent_settings_error",
    "local_extensions_error",
    "local_extensions_conflict",
    "local_extensions_readonly",
    "invalid_response",
    "invalid_request",
    "missing_device_credential",
    "client_update_storage_error",
    "client_update_install_failed",
    "client_update_not_downloaded",
    "client_update_unsupported_platform",
    "invalid_client_update",
    "forbidden",
    "rate_limited",
    "server_error",
    "upstream_connect_timeout",
    "upstream_connect_failed",
    "upstream_test_failed",
    "upstream_status",
  ]) {
    const text = errorText({ code, message: "local diagnostic" });

    expect(text).not.toBe("local diagnostic");
    expect(text).toMatch(/[\u4e00-\u9fa5]/);
  }
});

test("未识别的错误码回退到原始消息", () => {
  const error = { code: "code_from_a_newer_server", message: "future failure" };

  expect(errorText(error)).toBe("future failure");
});

test("已知错误码不再暴露原始诊断", () => {
  const error = {
    code: "network_error",
    message: "reqwest::Error { kind: Request }",
  };

  expect(errorText(error)).toContain("无法连接");
  expect(errorText(error)).not.toContain("reqwest");
});

test("校验失败只显示客户端固定文案，不再回显服务端句子", () => {
  const error = {
    code: "validation_failed",
    message: "模型 gpt-6-luna 不在供应商“GoToken 套餐”的模型清单里，请先移除",
  };

  expect(errorText(error)).toBe("请求内容不合法。");
  expect(errorText(error)).not.toContain("gpt-6-luna");
});

test("管理命令通知只使用文案层", () => {
  expect(source("composables/command/relay.ts")).toContain(
    "notifications.error(errorText(relayError)",
  );
  expect(source("composables/command/relay.ts")).not.toContain("errorReason");
});

test("旧版 HTTP 兜底消息仍转换为中文", () => {
  const error = {
    code: "http_status_only",
    message: "management API returned HTTP 404 Not Found",
  };

  expect(errorText(error)).toBe("管理服务未找到请求的管理接口（HTTP 404）。");
});

test("没有消息的未知错误给出兜底文案", () => {
  expect(errorText({ code: "unknown", message: "" })).toBe("发生了未知错误。");
});

test("旧版错误字符串只按行首已知码识别", () => {
  expect(
    toRelayError("network_error: unable to reach the relay management API"),
  ).toEqual({
    code: "network_error",
    message: "network_error: unable to reach the relay management API",
  });

  expect(toRelayError("request failed: not_found in cache")).toEqual({
    code: "unknown",
    message: "request failed: not_found in cache",
  });
});

test("活动记录错误码清单与客户端文案一一对应", () => {
  for (const code of [
    "upstream_status",
    "upstream_connect",
    "upstream_response_head_timeout",
    "upstream_invalid_response",
    "upstream_body",
    "upstream_error",
    "empty_stream",
    "stream_error",
    "downstream_disconnect",
    "activity_rollout_incomplete",
    "activity_interrupted",
  ]) {
    expect(activityErrorText(code)).toBeTruthy();
  }

  expect(activityErrorText("code_from_a_newer_server")).toBeNull();
  expect(activityErrorText(null)).toBeNull();
});

test("界面展示错误时统一使用文案层", () => {
  for (const path of [
    "pages/setup.vue",
    "composables/command/relay.ts",
    "composables/command/local.ts",
    "composables/clientUpdate.ts",
  ]) {
    expect(source(path)).toContain("errorText(");
  }
  expect(source("components/shell/ConnectionError.vue")).toContain(
    "无法连接服务地址",
  );
  expect(source("composables/extension/installJobs.ts")).toContain(
    "errorText(",
  );
  expect(source("components/activity/Table.vue")).toContain(
    "activityErrorText(",
  );
});
