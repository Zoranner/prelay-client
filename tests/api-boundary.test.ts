import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("Nuxt 管理页面只通过固定的 Tauri command 调用服务端", () => {
  const commands = source("composables/useRelayCommand.ts");

  for (const command of [
    "bootstrap",
    "relay_settings_get",
    "relay_settings_connect",
    "relay_settings_save",
    "providers_list",
    "catalog_providers_list",
    "providers_save",
    "providers_delete",
    "providers_ping",
    "providers_test_protocol",
    "endpoints_list",
    "endpoints_save",
    "endpoints_delete",
    "endpoints_regenerate_token",
    "stats_overview",
    "stats_activities",
    "stats_models",
    "stats_providers",
    "credential_rotate",
  ]) {
    expect(commands).toContain(`"${command}"`);
  }

  expect(commands).toContain("@tauri-apps/api/core");
  expect(commands).toContain("invoke");
});

test("仪表盘使用同一范围读取概览、趋势、模型和供应商统计", () => {
  const page = source("pages/index.vue");

  expect(page).toContain(
    'invokeCommand<StatsOverview>("stats_overview", range)',
  );
  expect(page).toContain('invokeCommand<ModelStats[]>("stats_models", range)');
  expect(page).toContain(
    'invokeCommand<ProviderStats[]>("stats_providers", range)',
  );
  expect(page).toContain(
    'invokeCommand<TokenUsageTimelinePoint[]>("stats_timeline", range)',
  );
  expect(page).not.toContain("查看活动");
});

test("全屏管理服务错误读取嵌套 Ref 的当前值", () => {
  const app = source("app.vue");

  expect(app).toContain(
    "const managementApiError = computed(() => managementApi.error.value);",
  );
  expect(app).toContain("managementApiError && canShowManagementError");
  expect(app).toContain("<Result");
  expect(app).toContain("{{ managementApiError.message }}");
});

test("Nuxt 页面不直连服务端或读取认证凭据", () => {
  for (const page of [
    "pages/index.vue",
    "pages/providers.vue",
    "pages/endpoints.vue",
    "pages/stats.vue",
    "pages/setup.vue",
    "pages/settings.vue",
  ]) {
    const content = source(page);
    expect(content).not.toContain("fetch(");
    expect(content).not.toContain("Authorization");
    expect(content).not.toContain("device-credential");
    expect(content).not.toContain("localStorage");
  }
});
