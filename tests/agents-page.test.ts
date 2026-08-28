import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("智能体页面只编排路由级状态，展示和设置由领域组件承担", () => {
  const page = source("pages/agents.vue");
  const shell = source("components/dashboard/DashboardShell.vue");
  const sidebar = source("components/agents/AgentSidebar.vue");
  const content = source("components/agents/AgentWorkspaceContent.vue");
  const drawer = source("components/agents/AgentSettingsDrawer.vue");

  expect(shell).toContain(
    '{ label: "智能体", path: "/agents", icon: "ph:robot" }',
  );
  expect(page).toContain('title="智能体"');
  expect(page).toContain("availableSectionOptions");
  expect(page).toContain("AgentSidebar");
  expect(page).toContain("AgentWorkspaceContent");
  expect(page).toContain("AgentSettingsDrawer");
  expect(page).not.toContain("ClaudeCode");
  expect(page).not.toContain("claudeCode");
  expect(sidebar).toContain("agent-client-list");
  expect(sidebar).toContain("agent-client-icon--monochrome");
  expect(content).toContain("@update:model-value");
  expect(page).toContain('value: "rules"');
  expect(page).toContain("插件");
  expect(page).toContain("MCP");
  expect(page).toContain("Skill");
  expect(content).not.toContain('icon="ph:gear-six"');
  expect(drawer).toContain("CodexSettingsForm");
  expect(drawer).toContain("ChatGptSettingsForm");
  expect(drawer).toContain("OpenCodeSettingsForm");
  expect(drawer).toContain("agent-settings-form");
});

test("规则编辑器独立负责输入、预览和滚动同步", () => {
  const rulesEditor = source("components/agents/AgentRulesEditor.vue");

  expect(rulesEditor).toContain("defineModel<string>");
  expect(rulesEditor).toContain("Textarea");
  expect(rulesEditor).toContain("MarkdownViewer");
  expect(rulesEditor).toContain("syncRulesScroll");
  expect(rulesEditor).toContain("onBeforeUnmount");
  expect(rulesEditor).toContain("agent-rules__editor");
  expect(rulesEditor).toContain("agent-rules__preview");
});

test("Codex 设置表单独立承载连接、执行和运行环境字段", () => {
  const form = source("components/agents/CodexSettingsForm.vue");

  expect(form).toContain("defineModel<CodexSettingsDraft>");
  expect(form).toContain("模型与接入");
  expect(form).toContain("执行与网络");
  expect(form).toContain("协作与记忆");
  expect(form).toContain("运行环境");
  expect(form).toContain("禁用响应存储");
  expect(form).toContain("最大并发智能体");
  expect(form).toContain("工作区网络访问");
  expect(form).toContain("Shell 环境继承");
  expect(form).toContain("Windows 沙箱");
  expect(form).toContain("customToken");
  expect(form).not.toContain('type="password"');
  expect(form).toContain("watch(");
  expect(form).toContain('customBaseUrl = ""');
});
