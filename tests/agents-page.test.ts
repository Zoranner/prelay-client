import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("智能体页左侧切换客户端，右侧顶部切换功能区", () => {
  const page = source("pages/agents.vue");
  const shell = source("components/dashboard/DashboardShell.vue");

  expect(shell).toContain('{ label: "智能体", path: "/agents", icon: "ph:robot" }');
  expect(page).toContain('title="智能体"');
  expect(page).toContain("activeClient");
  expect(page).toContain("agent-client-list");
  expect(page).toContain("agent-client-icon--monochrome");
  expect(page).toContain("@lobehub/icons-static-svg/icons/openai.svg");
  expect(page).toContain("@lobehub/icons-static-svg/icons/claudecode-color.svg");
  expect(page).toContain('class="agent-toolbar"');
  expect(page).toMatch(
    /class="agent-content"[\s\S]*agent-client-list[\s\S]*class="agent-main"[\s\S]*class="agent-toolbar"[\s\S]*v-model="activeSection"[\s\S]*variant="button"/,
  );
  expect(page).toContain("<template #header-actions>");
  expect(page).toContain('variant="primary"');
  expect(page).toContain('icon="ph:arrows-clockwise"');
  expect(page).toContain("刷新");
  expect(page).toContain('aria-label="编辑设置"');
  expect(page).toContain("插件");
  expect(page).toContain("MCP");
  expect(page).toContain("Skill");
  expect(page).toContain("agent-main");
  expect(page).toContain('<div :key="activeClient" class="agent-main">');
  expect(page).toContain("模型与接入");
  expect(page).toContain("执行与网络");
  expect(page).toContain("协作与记忆");
  expect(page).toContain("运行环境");
  expect(page.indexOf("模型与接入")).toBeLessThan(
    page.indexOf("执行与网络"),
  );
  expect(page.indexOf("执行与网络")).toBeLessThan(
    page.indexOf("协作与记忆"),
  );
  expect(page.indexOf("协作与记忆")).toBeLessThan(
    page.indexOf("运行环境"),
  );
  expect(page).toContain("agent-settings__group");
  expect(page).toContain("agent-settings__label");
  expect(page).toContain("agent-settings-form");
  expect(page).toContain("agent-rules");
  expect(page).toMatch(/\.agent-rules\s*{[\s\S]*?grid-template-columns/);
  expect(page).not.toContain("<strong>");
  expect(page).toContain("font-size: 15px");
  expect(page).not.toMatch(
    /\.agent-settings__group \+ \.agent-settings__group\s*{[\s\S]*?border-top/,
  );
  expect(page).not.toMatch(
    /\.agent-settings__rows\s*{[\s\S]*?border-top/,
  );
  expect(page).not.toMatch(
    /\.agent-settings__row\s*{[\s\S]*?border-bottom/,
  );
  expect(page).toContain("接入点");
  expect(page).toContain('label: "规则"');
  expect(page).not.toContain("AGENTS.md");
  expect(page).not.toContain("CLAUDE.md");
  expect(page).toContain("禁用响应存储");
  expect(page).toContain("最大并发智能体");
  expect(page).toContain("子智能体最大深度");
  expect(page).toContain("单项任务时限");
  expect(page).toContain("工作区网络访问");
  expect(page).toContain("Shell 环境继承");
  expect(page).toContain("Windows 沙箱");
  expect(page).toContain('value: "all", label: "全部继承"');
  expect(page).toContain('value: "core", label: "仅基础环境"');
  expect(page).toContain('value: "none", label: "不继承"');
  expect(page).toContain('value: "unelevated", label: "非提升"');
  expect(page).toContain('value: "elevated", label: "提升"');
  expect(page).not.toContain(
    'Input v-model="agentConfiguration.codex.shellEnvironmentInherit"',
  );
  expect(page).not.toContain(
    'Input v-model="agentConfiguration.codex.windowsSandbox"',
  );
  expect(page).toContain("长期记忆");
  expect(page).toContain("目标管理");
  expect(page).toContain("工作区依赖");
  expect(page).not.toContain("Responses WebSocket v2");
  expect(page).not.toContain("远程控制");
  expect(page).not.toContain("RMCP 客户端");
  expect(page).toContain("<AgentItemList");
  expect(page).toContain('"endpoints_list"');
  expect(page).toContain("claudeCodeConnection");
  expect(page).toContain("endpointToken");
  expect(page).toMatch(
    /class="agent-settings__row">\s*<span class="agent-settings__label">执行权限<\/span>[\s\S]*?<RadioGroup/,
  );
  expect(page).toMatch(
    /class="agent-settings__row">\s*<span class="agent-settings__label">工具权限<\/span>[\s\S]*?<RadioGroup/,
  );
  expect(page).not.toContain("@media (max-width: 760px)");
});

test("自定义 Codex API Key 会回显到可查看的输入框", () => {
  const page = source("pages/agents.vue");

  expect(page).toContain('v-model="settingsDraft.codex.customToken"');
  expect(page).not.toContain('type="password"');
});
