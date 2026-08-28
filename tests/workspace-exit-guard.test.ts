import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";

const guardPath = new URL(
  "../app/composables/useWorkspaceExitGuard.ts",
  import.meta.url,
);

test("切换服务地址前由工作区退出守卫统一判断活动编辑面", () => {
  expect(existsSync(guardPath)).toBe(true);

  const source = readFileSync(guardPath, "utf8");
  expect(source).toContain("register");
  expect(source).toContain("requestExit");
  expect(source).toContain('"allow"');
  expect(source).toContain('"discard"');
  expect(source).toContain('"blocked"');
  expect(source).toContain("[...entries.value].reverse()");
  expect(source).toContain("for (const current of entriesToClose)");
});

test("智能体设置放弃修改时恢复所有客户端草稿", () => {
  const source = readFileSync(
    new URL("../app/composables/useAgentSettings.ts", import.meta.url),
    "utf8",
  );

  expect(source).toContain("function discard()");
  expect(source).toContain('copyAgentClientSettings(configuration, draft, "codexCli")');
  expect(source).toContain('copyAgentClientSettings(configuration, draft, "chatgpt")');
  expect(source).toContain('copyAgentClientSettings(configuration, draft, "openCode")');
  expect(source).not.toContain("claudeCode");
});

test("状态栏和有草稿的编辑面都接入工作区退出守卫", () => {
  const source = (path: string) =>
    readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

  expect(source("components/dashboard/DashboardStatusbar.vue")).toContain(
    "useWorkspaceExitGuard",
  );
  expect(source("components/settings/DesktopPreferencesDialog.vue")).toContain(
    "useWorkspaceExitGuard",
  );
  expect(source("pages/providers.vue")).toContain("useWorkspaceExitGuard");
  expect(source("pages/endpoints.vue")).toContain("useWorkspaceExitGuard");
  expect(source("pages/agents.vue")).toContain("useWorkspaceExitGuard");
});
