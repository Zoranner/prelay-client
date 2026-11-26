import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import { getProviderOperationFeedback } from "../app/utils/provider/operations";

const providerFormDomain = () =>
  ["../app/components/providers/Form.vue", "../app/composables/providerForm.ts"]
    .map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
    .join("\n");

test("上游 200 但 ok 为 false 时将供应商操作显示为失败，并保留诊断指标", () => {
  const feedback = getProviderOperationFeedback({
    ok: false,
    protocol: "openai",
    error_code: "upstream_connect_failed",
    error: "upstream connect failed",
    latency_ms: 840,
    first_token_ms: 380,
  });

  expect(feedback.success).toBe(false);
  expect(feedback.message).toBe("无法连接上游服务。");
  expect(feedback.metrics).toBe("延迟 840 ms；首 Token 380 ms");
});

test("供应商操作失败没有码时使用固定文案", () => {
  const feedback = getProviderOperationFeedback({
    ok: false,
    protocol: "openai",
    error: "upstream returned status=500",
    latency_ms: 120,
    first_token_ms: null,
  });

  expect(feedback.success).toBe(false);
  expect(feedback.message).toBe("协议测试失败。");
});

test("协议测试成功时使用服务端返回的协议字段", () => {
  const feedback = getProviderOperationFeedback({
    ok: true,
    protocol: "responses",
    latency_ms: 240,
    first_token_ms: 110,
    error: null,
  });

  expect(feedback.success).toBe(true);
  expect(feedback.message).toBe("Responses 协议测试完成。");
  expect(feedback.metrics).toBe("延迟 240 ms；首 Token 110 ms");
});

test("Nuxt 供应商操作 DTO 只声明 Tauri 实际返回的字段", () => {
  const source = readFileSync(
    new URL("../app/utils/provider/operations.ts", import.meta.url),
    "utf8",
  );

  expect(source).toContain("protocol: string | null;");
  expect(source).not.toContain("message?: string | null;");
  expect(source).toContain("latency_ms: number | null;");
  expect(source).toContain("first_token_ms: number | null;");
  expect(source).toContain("error_code?: string | null;");
  expect(source).toContain("error: string | null;");
  expect(source).not.toContain("models");
});

test("供应商页面按供应商 ID 执行连通性检查，不传 API Key", () => {
  const page = readFileSync(
    new URL("../app/pages/providers.vue", import.meta.url),
    "utf8",
  );
  const command = readFileSync(
    new URL("../src-tauri/src/commands/providers.rs", import.meta.url),
    "utf8",
  );

  expect(page).not.toContain("notifications.notify");
  expect(page).toContain('"providers_ping"');
  expect(page).toContain("providerId: provider.id");
  expect(page).not.toMatch(/providers_ping[\s\S]{0,180}api_key/);
  expect(command).toContain('format!("/api/providers/{provider_id}/ping")');
  expect(page).not.toContain('"providers_discover_models"');
  expect(page).toContain('"providers_test_protocol"');
});

test("供应商表单内的协议测试也发送全局通知并保留指标", () => {
  const form = providerFormDomain();

  expect(form).toContain("getProviderOperationFeedback");
  expect(form).toContain("notifications.notify");
  expect(form).toContain('semantic: feedback.success ? "success" : "error"');
  expect(form).toContain('message: feedback.metrics ?? ""');
});

test("供应商表单不通过上游发现覆盖目录模型", () => {
  const form = providerFormDomain();

  expect(form).not.toContain("requestDiscovery");
  expect(form).not.toContain("模型已获取");
  expect(form).not.toContain("models.value = result.models ?? [];");
});

test("供应商表单不提供模型发现失败或手工添加提示", () => {
  const form = providerFormDomain();

  expect(form).not.toContain("模型列表暂不可用");
  expect(form).not.toContain("可手工添加模型后保存供应商");
});

test("供应商保存命令发送启用清单", () => {
  const page = readFileSync(
    new URL("../app/pages/providers.vue", import.meta.url),
    "utf8",
  );
  const command = readFileSync(
    new URL("../src-tauri/src/commands/providers.rs", import.meta.url),
    "utf8",
  );

  expect(page).toContain("models: payload.models");
  expect(command).toContain("pub models: Option<Vec<String>>");
  expect(command).toContain("models: input.models");
});

test("供应商前端只有一个读模型，列表与编辑共用", () => {
  const relay = readFileSync(
    new URL("../app/types/relay.ts", import.meta.url),
    "utf8",
  );
  const quota = readFileSync(
    new URL("../app/types/quota.ts", import.meta.url),
    "utf8",
  );

  expect(relay).toContain(
    'export type ProviderVisibility = "private" | "selected" | "all";',
  );
  expect(relay).toContain("export interface Provider {");
  expect(relay).not.toContain("ProviderListItem");
  expect(relay).toContain("owner_identity_id: string;");
  expect(relay).toContain("owner_display_name: string;");
  expect(relay).toContain("credential: ProviderCredential | null;");
  expect(relay).not.toContain("can_manage");
  expect(relay).not.toContain("selected_identity_ids");
  expect(relay).toContain("export interface ProviderUsage");
  expect(relay).toContain("export interface ProviderUsageUser");
  expect(relay).toContain("export interface IdentityDirectoryEntry");
  expect(quota).toContain("export interface ProviderSharing");
  expect(quota).toContain("export interface ShareQuotaRule");
  expect(quota).toContain("export interface ProviderQuotaResponse");

  expect(relay.match(/export interface Provider \{/g)?.length).toBe(1);

  const readModel = relay.slice(
    relay.indexOf("export interface Provider {"),
    relay.indexOf("export interface ProviderUsage"),
  );
  expect(readModel).not.toContain("api_key");
  expect(readModel).toContain("credential: ProviderCredential | null;");
});
