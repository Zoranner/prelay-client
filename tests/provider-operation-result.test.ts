import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import { getProviderOperationFeedback } from "../app/utils/providerOperations";

test("上游 200 但 ok 为 false 时将供应商操作显示为失败，并保留诊断指标", () => {
  const feedback = getProviderOperationFeedback({
    ok: false,
    protocol: "openai",
    error: "上游认证失败",
    latency_ms: 840,
    first_token_ms: 380,
    models: null,
  });

  expect(feedback.success).toBe(false);
  expect(feedback.message).toBe("上游认证失败");
  expect(feedback.metrics).toBe("延迟 840 ms；首 Token 380 ms");
});

test("模型发现成功时保留服务端返回的模型列表", () => {
  const feedback = getProviderOperationFeedback({
    ok: true,
    protocol: null,
    latency_ms: null,
    first_token_ms: null,
    error: null,
    models: ["deepseek-chat", "deepseek-reasoner"],
  });

  expect(feedback.success).toBe(true);
  expect(feedback.message).toBe("发现模型：deepseek-chat、deepseek-reasoner");
});

test("协议测试成功时使用服务端返回的协议字段", () => {
  const feedback = getProviderOperationFeedback({
    ok: true,
    protocol: "responses",
    latency_ms: 240,
    first_token_ms: 110,
    error: null,
    models: null,
  });

  expect(feedback.success).toBe(true);
  expect(feedback.message).toBe("Responses 协议测试完成。");
  expect(feedback.metrics).toBe("延迟 240 ms；首 Token 110 ms");
});

test("Nuxt 供应商操作 DTO 只声明 Tauri 实际返回的字段", () => {
  const source = readFileSync(
    new URL("../app/utils/providerOperations.ts", import.meta.url),
    "utf8",
  );

  expect(source).toContain("protocol: string | null;");
  expect(source).not.toContain("message?: string | null;");
  expect(source).toContain("latency_ms: number | null;");
  expect(source).toContain("first_token_ms: number | null;");
  expect(source).toContain("error: string | null;");
  expect(source).toContain("models: string[] | null;");
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
  expect(page).toContain('"providers_discover_models"');
  expect(page).toContain('"providers_test_protocol"');
});

test("供应商表单内的协议测试也发送全局通知并保留指标", () => {
  const form = readFileSync(
    new URL(
      "../app/components/providers/ProviderForm.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(form).toContain("getProviderOperationFeedback");
  expect(form).toContain("notifications.notify");
  expect(form).toContain('type: feedback.success ? "success" : "danger"');
  expect(form).toContain('message: feedback.metrics ?? ""');
});

test("供应商表单以模型发现结果覆盖本地清单并发送结果通知", () => {
  const form = readFileSync(
    new URL(
      "../app/components/providers/ProviderForm.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(form).toContain("notifications.success(");
  expect(form).toContain('title: "模型已获取"');
  expect(form).toContain("models.value = result.models ?? [];");
  expect(form).not.toContain("本次新增");
});
