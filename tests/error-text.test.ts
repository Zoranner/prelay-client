import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import {
  errorDetail,
  errorReason,
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
    "invalid_response",
    "invalid_request",
    "missing_device_credential",
    "client_update_storage_error",
    "client_update_install_failed",
    "client_update_not_downloaded",
    "client_update_unsupported_platform",
    "invalid_client_update",
  ]) {
    const text = errorText({ code, message: "local diagnostic" });

    expect(text).not.toBe("local diagnostic");
    expect(text).toMatch(/[\u4e00-\u9fa5]/);
  }
});

test("未识别的错误码回退到原始消息并作为详情展示", () => {
  const error = { code: "code_from_a_newer_server", message: "future failure" };

  expect(errorText(error)).toBe("future failure");
  expect(errorDetail(error)).toBe("future failure");
});

test("已知错误码不再暴露原始诊断", () => {
  const error = {
    code: "network_error",
    message: "reqwest::Error { kind: Request }",
  };

  expect(errorText(error)).toContain("无法连接");
  expect(errorDetail(error)).toBeNull();
});

test("校验失败把服务端给出的具体原因交回界面", () => {
  const reason =
    "模型 gpt-6-luna 不在供应商“GoToken 套餐”的模型清单里，请先移除或改选后再保存";

  expect(errorReason({ code: "validation_failed", message: reason })).toBe(
    reason,
  );
  expect(errorReason({ code: "validation_failed", message: "   " })).toBeNull();
  expect(
    errorReason({ code: "validation_failed", message: "请求内容不合法。" }),
  ).toBeNull();
  expect(
    errorReason({ code: "internal", message: "Internal server error" }),
  ).toBeNull();
  expect(
    errorReason({ code: "network_error", message: "reqwest::Error" }),
  ).toBeNull();
});

test("管理命令通知优先展示具体原因", () => {
  expect(source("composables/useRelayCommand.ts")).toContain(
    "errorReason(relayError) ?? errorText(relayError)",
  );
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

test("错误字符串仍按已知错误码识别", () => {
  expect(
    toRelayError("network_error: unable to reach the relay management API"),
  ).toEqual({
    code: "network_error",
    message: "network_error: unable to reach the relay management API",
  });
});

test("界面展示错误时统一使用文案层", () => {
  for (const path of [
    "app.vue",
    "composables/useRelayCommand.ts",
    "composables/useLocalCommand.ts",
    "composables/useClientUpdate.ts",
  ]) {
    expect(source(path)).toContain("errorText(");
  }
  expect(source("components/extensions/ExtensionInstallDrawer.vue")).toContain(
    "errorText(",
  );
});
