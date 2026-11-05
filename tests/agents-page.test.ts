import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("智能体页面只编排路由级状态，展示和设置由领域组件承担", () => {
  const page = source("pages/agents.vue");
  const shell = source("components/dashboard/frame/Shell.vue");
  const sidebar = source("components/agents/workspace/Sidebar.vue");
  const clientContent = source("components/agents/workspace/Content.vue");
  const extensionContent = source("components/extensions/library/Content.vue");
  const selection = source("composables/agent/pageSelection.ts");
  const drawer = source("components/agents/settings/Drawer.vue");
  const installDrawer = source("components/extensions/install/Drawer.vue");

  expect(shell).toContain(
    '{ label: "智能体", path: "/agents", icon: "ph:robot" }',
  );
  expect(page).toContain('title="智能体"');
  expect(page).toContain("availableSectionOptions");
  expect(page).toContain("AgentSidebar");
  expect(page).toContain("AgentClientContent");
  expect(page).toContain("ExtensionLibraryContent");
  expect(page).toContain("AgentSettingsDrawer");
  expect(sidebar).toContain("agent-client-list");
  expect(sidebar).toContain("AgentClientIcon");
  expect(sidebar).not.toContain("agent-client-icon--monochrome");
  expect(clientContent).toContain("@update:model-value");
  expect(clientContent).toContain("activeSection === 'plugin'");
  expect(clientContent).toContain("AgentHookList");
  expect(clientContent).not.toContain("showItemStatus");
  expect(extensionContent).toContain("@update:model-value");
  expect(extensionContent).toContain("<ExtensionCatalogTable");
  expect(selection).toContain("agentSectionOptions");
  expect(selection).toContain("AgentSectionOption");
  expect(clientContent).not.toContain('icon="ph:gear-six"');
  expect(drawer).toContain("CodexSettingsForm");
  expect(drawer).toContain("ChatGptSettingsForm");
  expect(drawer).not.toContain("OpenCodeSettingsForm");
  expect(drawer).not.toContain("ClaudeCodeSettingsForm");
  expect(installDrawer).not.toContain("claudeCode");
  expect(drawer).toContain("agent-settings-form");
});

test("进入智能体页面时默认选中客户端列表第一项", () => {
  const selection = source("composables/agent/pageSelection.ts");

  expect(selection).toContain(
    'const initialClient = agentClients.value[0]?.client ?? "chatgpt";',
  );
  expect(selection).toContain(
    "const activeWorkspace = ref<AgentWorkspace>(initialClient);",
  );
  expect(selection).toContain(
    'const fallback = agentClients.value[0]?.client ?? "chatgpt";',
  );
  expect(selection).not.toContain(
    'const activeWorkspace = ref<AgentWorkspace>("codexCli");',
  );
  expect(selection).not.toContain(
    'const fallback =\n        clientStatuses.value.find((status) => status.installed)?.client ??\n        "codexCli";',
  );
});

test("接入点目录或服务地址晚于本机配置返回时重新反推智能体接入点", () => {
  const settings = source("composables/agent/settings.ts");

  expect(settings).toContain("endpointOptionsForClient");
  expect(settings).toContain("watch(");
  expect(settings).toContain("if (codexCli) hydrate(codexCli);");
  expect(settings).toContain("if (chatgpt) hydrate(chatgpt);");
  expect(settings).not.toContain("openCode");
});

test("智能体刷新按钮是带图标的主操作并保留加载反馈", () => {
  const page = source("pages/agents.vue");

  expect(page).toMatch(
    /<Button[\s\S]*semantic="primary"[\s\S]*variant="solid"[\s\S]*icon="ph:arrows-clockwise"[\s\S]*:disabled="clientStatusesLoading"[\s\S]*@click="refreshAgentClients"/,
  );
});

test("规则编辑器独立负责输入、预览和滚动同步", () => {
  const rulesEditor = source("components/agents/workspace/RulesEditor.vue");

  expect(rulesEditor).toContain("defineModel<string>");
  expect(rulesEditor).toContain("Textarea");
  expect(rulesEditor).toContain("MarkdownViewer");
  expect(rulesEditor).toContain("syncRulesScroll");
  expect(rulesEditor).toContain("onBeforeUnmount");
  expect(rulesEditor).toContain("agent-rules__editor");
  expect(rulesEditor).toContain("agent-rules__preview");
  expect(rulesEditor).not.toContain("padding: 0 0 var(--spacing-lg);");
  expect(source("components/agents/workspace/Content.vue")).not.toContain(
    "padding-top: var(--spacing-md);",
  );
});

test("Codex 设置表单独立承载连接、执行和运行环境字段", () => {
  const form = source("components/agents/settings/forms/Codex.vue");

  expect(form).toContain("defineModel<CodexSettingsDraft>");
  expect(form).toContain("模型与接入");
  expect(form).toContain("执行与网络");
  expect(form).toContain("协作与记忆");
  expect(form).toContain("运行环境");
  expect(form).toContain("最大并发智能体");
  expect(form).toContain("工作区网络访问");
  expect(form).toContain("Shell 环境继承");
  expect(form).toContain("Windows 沙箱");
  expect(form).not.toContain("customToken");
  expect(form).not.toContain('type="password"');
  expect(form).not.toContain("customBaseUrl");
});
