import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("桌面客户端按参考工程采用 shell、dashboard 和业务域目录", () => {
  const app = source("app.vue");
  const shell = source("components/dashboard/DashboardShell.vue");
  const styles = source("assets/css/main.css");

  expect(app).toContain("DashboardShell");
  expect(shell).toContain('from "@stellar/ui"');
  expect(shell).toContain("Sidebar");
  expect(styles).toContain('@import "@stellar/ui/styles"');
  expect(styles).not.toContain("tokens.css");
  expect(styles).not.toContain("business.css");
});

test("供应商和接入点页面通过 Stellar UI 的 PanelSection、Table 和 Drawer 组合", () => {
  for (const page of ["pages/providers.vue", "pages/endpoints.vue"]) {
    const content = source(page);
    expect(content).toContain('from "@stellar/ui"');
    expect(content).toContain("PanelSection");
    expect(content).toContain(":visible=");
    expect(content).toContain("@update:visible");
  }

  for (const component of [
    "components/providers/ProviderList.vue",
    "components/endpoints/EndpointList.vue",
  ]) {
    const content = source(component);
    expect(content).toContain('from "@stellar/ui"');
    expect(content).toContain("<Table");
    expect(content).not.toContain("~/components/display/Table.vue");
  }
});

test("供应商和接入点表单保留历史业务字段，并用 Stellar UI 表单组件承载", () => {
  const providerForm = source("components/providers/ProviderForm.vue");
  const endpointForm = source("components/endpoints/EndpointForm.vue");

  expect(providerForm).toContain("PROVIDER_TEMPLATE_GROUPS");
  expect(providerForm).toContain("获取模型");
  expect(providerForm).toContain("protocolBaseUrls");
  expect(providerForm).toContain("<Input");
  expect(providerForm).toContain("<Select");
  expect(endpointForm).toContain("newModelForm");
  expect(endpointForm).toContain("新增模型");
  expect(endpointForm).toContain("<Select");
});

test("编辑抽屉的表单章节不再嵌套边框", () => {
  const providerForm = source("components/providers/ProviderForm.vue");
  const endpointForm = source("components/endpoints/EndpointForm.vue");

  for (const form of [providerForm, endpointForm]) {
    expect(form).toMatch(/\.\w+-form\s*\{\s*padding: var\(--spacing-lg\);/);
    expect(form).not.toMatch(/\.form-section\s*\{[^}]*border:/s);
  }
});
