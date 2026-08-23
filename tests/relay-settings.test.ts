import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import { toRelayError } from "../app/utils/errors";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("网络错误字符串会保留为 network_error", () => {
  expect(
    toRelayError("network_error: unable to reach the relay management API"),
  ).toEqual({
    code: "network_error",
    message: "network_error: unable to reach the relay management API",
  });
});

test("服务地址只在全屏接入点页通过连接命令确认", () => {
  const settings = source("composables/useRelaySettings.ts");
  const setup = source("pages/setup.vue");
  const settingsPage = source("pages/settings.vue");
  const middleware = source("middleware/setup.global.ts");
  const statusbar = source("components/workbench/WorkbenchStatusbar.vue");

  expect(settings).toContain('"relay_settings_connect"');
  expect(setup).toContain("await settings.connect(relayUrl.value)");
  expect(setup).not.toContain("await settings.save(relayUrl.value)");
  expect(settingsPage).not.toContain("useRelaySettings");
  expect(settingsPage).not.toContain("relay_settings_connect");
  expect(settingsPage).not.toContain("管理服务地址");
  expect(settingsPage).toContain("useDesktopPreferencesDialog");
  expect(settingsPage).toContain("desktopPreferencesDialog.open");
  expect(settingsPage).toContain('navigateTo("/")');
  expect(middleware).toContain('to.query.change !== "1"');
  expect(statusbar).toContain('navigateTo("/setup?change=1")');
  expect(setup).toContain('route.query.change === "1"');
  expect(setup).toContain('icon="ph:arrow-left"');
  expect(setup).toContain('navigateTo("/")');
  expect(setup).toContain('class="setup-form__eyebrow"');
  expect(setup).toContain(">连接管理服务<");
  expect(setup).not.toContain("<h1>连接管理服务</h1>");
  expect(setup).not.toContain(">Prelay<");
});

test("切换管理服务时回填当前保存的地址", () => {
  const setup = source("pages/setup.vue");

  expect(setup).toContain(
    'isChangingAddress.value ? (settings.relayUrl.value ?? "") : "",',
  );
});
