import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

test("通知由 Stellar UI 的全局容器承载，不插入页面和抽屉内容流", () => {
  const app = readFileSync(new URL("../app/app.vue", import.meta.url), "utf8");
  const surfaces = [
    "../app/pages/index.vue",
    "../app/pages/endpoints.vue",
    "../app/pages/providers.vue",
    "../app/pages/settings.vue",
    "../app/pages/setup.vue",
    "../app/pages/stats.vue",
    "../app/components/endpoints/EndpointForm.vue",
    "../app/components/providers/ProviderForm.vue",
  ];

  for (const surface of surfaces) {
    expect(
      readFileSync(new URL(surface, import.meta.url), "utf8"),
    ).not.toContain("notice--danger");
  }

  expect(app).toContain('from "stellar-ui"');
  expect(app).toContain("NotificationContainer");
  expect(app).toContain('position="top-right"');
  expect(app).toContain(':max="3"');
  expect(app).not.toContain("NotificationViewport");
});

test("客户端不保留自定义通知实现", () => {
  const composable = readFileSync(
    new URL("../app/composables/useRelayCommand.ts", import.meta.url),
    "utf8",
  );

  expect(composable).toContain("useNotification");
  expect(composable).not.toContain("useNotifications");
});
