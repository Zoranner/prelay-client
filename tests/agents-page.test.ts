import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("智能体页保留客户端与功能区入口，并将展示职责拆分为子组件", () => {
  const page = source("pages/agents.vue");
  const shell = source("components/dashboard/DashboardShell.vue");

  expect(shell).toContain(
    '{ label: "智能体", path: "/agents", icon: "ph:robot" }',
  );
  expect(page).toContain('title="智能体"');
  expect(page).toContain("activeClient");
  expect(page).toContain("agent-client-list");
  expect(page).toContain("agent-client-icon--monochrome");
  expect(page).toContain('from "~/components/agents/AgentRulesEditor.vue"');
  expect(page).toContain('from "~/components/agents/CodexSettingsForm.vue"');
  expect(page).toContain(
    'from "~/components/agents/ClaudeCodeSettingsForm.vue"',
  );
  expect(page).toContain(
    '<AgentRulesEditor v-model="rulesDraft[activeClient]" />',
  );
  expect(page).toContain("<CodexSettingsForm");
  expect(page).toContain("<ClaudeCodeSettingsForm");
  expect(page).toContain('icon="ph:sliders-horizontal"');
  expect(page).toContain('aria-label="配置"');
  expect(page).toContain(
    ">\n                  配置\n                </Button>",
  );
  expect(page).not.toContain('icon="ph:gear-six"');
  expect(page).toContain('label: "规则"');
  expect(page).toContain("插件");
  expect(page).toContain("MCP");
  expect(page).toContain("Skill");
  expect(page).toContain("agent_settings_save");
  expect(page).toContain("claudeCodeConnection");
  expect(page).toContain("endpointToken");
  expect(page).toContain("function discardSettingsDraft()");
  expect(page).toContain("workspaceExit.register");
  expect(page).not.toContain("function syncRulesScroll");
  expect(page).not.toContain("function updateCodexNumber");
  expect(page).not.toContain("agent-settings__group");
  expect(page).not.toContain("agent-rules__editor");
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

test("Claude Code 设置表单独立承载模型和工具权限字段", () => {
  const form = source("components/agents/ClaudeCodeSettingsForm.vue");

  expect(form).toContain("defineModel<ClaudeCodeSettingsDraft>");
  expect(form).toContain("Opus 模型");
  expect(form).toContain("Sonnet 模型");
  expect(form).toContain("Haiku 模型");
  expect(form).toContain("子智能体模型");
  expect(form).toContain("工具权限");
  expect(form).toContain("RadioGroup");
});
