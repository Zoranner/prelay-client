import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("智能体页分别管理 Codex CLI、ChatGPT 与 OpenCode", () => {
  const app = source("app.vue");
  const navigation = source("components/dashboard/DashboardShell.vue");
  const page = source("pages/agents.vue");
  const workspace = source("composables/useAgentWorkspace.ts");
  const list = source("components/agents/AgentItemList.vue");
  const rulesEditor = source("components/agents/AgentRulesEditor.vue");

  expect(navigation).toContain('label: "智能体"');
  expect(navigation).toContain('path: "/agents"');
  expect(app).toContain("useAgentWorkspace");
  expect(app).toContain("void agentWorkspace.refreshClientStatuses()");
  expect(page).toContain("useAgentWorkspace");
  expect(page).toContain("agentWorkspace.refreshClient(activeClient.value)");
  expect(page).toContain("agentWorkspace.refreshClientStatuses()");
  expect(page).not.toContain('"agents_list"');
  expect(workspace).toContain('"agents_status"');
  expect(workspace).toContain('"agent_items_get"');
  expect(workspace).not.toContain('"agents_list"');
  expect(workspace).toContain('"agent_settings_get"');
  expect(workspace).toContain('"agent-workspace-client-statuses"');
  expect(workspace).toContain("const clientStatusesLoaded = useState");
  expect(workspace).toContain("const clientItems = useState");
  expect(page).toContain("workspaceExit.register");
  expect(page).toContain("void loadAgentPage()");
  expect(page).toContain("function refreshAgentClients()");
  expect(page).toContain("@click=\"refreshAgentClients\"");
  expect(page).toContain(':disabled="clientStatusesLoading"');
  expect(page).toContain("clientStatusesLoading ? 'ph:circle-notch' : 'ph:arrows-clockwise'");
  expect(page).toContain("'agent-refresh-icon--loading': clientStatusesLoading");
  expect(page).toContain("刷新");
  expect(page).toContain('from "~/components/agents/AgentItemList.vue"');
  expect(source("utils/agentClient.ts")).toContain("@lobehub/icons-static-svg");
  expect(page).toContain("activeClient");
  expect(page).toContain("agentClients");
  expect(page).toContain("agentClientDefinitions.map");
  expect(page).toContain("isClientInstalled");
  expect(page).toContain("agent-client-icon--uninstalled");
  expect(page).toMatch(
    /'agent-client-icon--loading':\s*isAgentStatusLoading\(\),/,
  );
  expect(page).toContain("return clientStatusesLoading.value;");
  expect(page).toContain('class="agent-client-loading"');
  expect(page).not.toContain('class="agent-client-loading-icon"');
  expect(page).toMatch(/icon="ph:circle-notch"\s+size="28"/);
  expect(page).toContain("Loading,");
  expect(page).toContain('text="正在读取智能体设置..."');
  expect(page).not.toContain("agent-main-loading");
  expect(page).not.toContain("agent-loading__icon");
  expect(page).toContain(
    ".agent-client-loading {\n  position: absolute;\n  inset: 0;\n  display: grid;\n  place-items: center;\n  pointer-events: none;\n  color: var(--st-text-primary);\n  animation: agent-loading-spin 800ms linear infinite;",
  );
  expect(page).not.toContain(
    ".agent-loading__icon {\n  color: var(--st-text-primary);",
  );
  expect(page).toContain("clientStatuses.value");
  expect(page).toContain("clientItems.value[activeClient.value]?.items");
  expect(page).toContain("clientStatusesLoaded.value && status?.installed");
  expect(workspace).toContain("clientStatusesLoaded.value = false;");
  expect(page).toContain('const rulesDraft = reactive({ codexCli: "", chatgpt: "", openCode: "" })');
  expect(page).not.toContain("claudeCode");
  expect(page).not.toContain("ClaudeCode");
  expect(page).toContain("activeClient === 'chatgpt'");
  expect(page).toContain('from "~/components/agents/ChatGptSettingsForm.vue"');
  expect(page).toContain('from "~/components/agents/OpenCodeSettingsForm.vue"');
  expect(page).toContain('v-model="rulesDraft.openCode"');
  expect(page).not.toContain("暂不提供本地设置管理");
  expect(page).toContain(
    "function replaceRulesDraft(client: AgentClient, rules: string)",
  );
  expect(page).toContain("rulesDraft[client] = rules");
  expect(page).toContain(
    'replaceRulesDraft("codexCli", agentConfiguration.codexCli.rules)',
  );
  expect(page).toContain(
    'replaceRulesDraft("chatgpt", agentConfiguration.chatgpt.rules)',
  );
  expect(page).toContain(
    'replaceRulesDraft("openCode", agentConfiguration.openCode.rules)',
  );
  expect(page).not.toContain(
    "rulesDraft.codex = agentConfiguration.codex.rules",
  );
  expect(page).not.toContain(
    "rulesDraft.claudeCode = agentConfiguration.claudeCode.rules",
  );
  expect(workspace).toContain('"agent_settings_get"');
  expect(workspace).toContain("{ notify: false, trackPending: false }");
  expect(page).toContain("const rulesSaving = ref(false)");
  expect(page).toContain("rulesSaving.value = true");
  expect(page).toContain("rulesSaving.value = false");
  expect(page).not.toContain("function syncRulesScroll");
  expect(rulesEditor).toContain("function syncRulesScroll");
  expect(rulesEditor).toContain("void nextTick().then(bindScroll)");
  expect(page).toContain(
    'rulesSaving.value ? "blocked" : rulesDirty.value ? "discard" : "allow"',
  );
  expect(page).not.toContain(
    'agentsPending.value ? "blocked" : rulesDirty.value ? "discard" : "allow"',
  );
  expect(page).not.toContain("const clients: Array<{ client: AgentClient;");
  expect(page).toContain("未检测到本机安装");
  expect(page).toContain("activeKind");
  expect(page).toContain('value: "plugin"');
  expect(page).toContain('value: "mcp"');
  expect(page).toContain('value: "skill"');
  expect(page).toContain("useConfirm");
  expect(page).toContain("uninstallAgentItem");
  expect(page).toContain('"agents_remove"');
  expect(page).toContain('@uninstall="uninstallAgentItem"');
  expect(page).toContain('<List class="agent-client-list"');
  expect(page).toContain("<ListItem");
  expect(page).toContain(':active="activeWorkspace === client.client"');
  expect(page).toContain('v-model="activeSection"');
  expect(page).toContain('variant="button"');
  expect(page).not.toContain("item-toolbar");
  expect(page).not.toContain("<Sidebar");
  expect(page).not.toContain("<Tabs");
  expect(page).not.toContain(':extra="String(client.items.length)"');
  expect(page).toContain("agent-client-icon--monochrome");
  expect(page).toContain("availableSectionOptions");
  expect(page).toContain("'agent-client-icon--monochrome': client.monochrome");
  expect(source("utils/agentClient.ts")).toContain(
    '{ client: "openCode", label: "OpenCode", icon: openCodeIcon, configurable: true, monochrome: true, sections: ["rules", "plugin", "mcp", "skill"] }',
  );
  expect(page).toContain('class="agent-client-icon-frame"');
  expect(page).toContain("border-radius: var(--radius-md)");
  expect(page).toContain(
    ".agent-client-list :deep(.st-list-item > div:first-child) {\n  display: flex;\n  width: 40px;",
  );
  expect(page).toContain("filter: var(--pr-monochrome-icon-filter)");
  expect(page).toContain(
    "filter: var(--pr-monochrome-icon-filter) grayscale(1)",
  );
  expect(source("assets/css/main.css")).toContain("html.light");
  expect(source("assets/css/main.css")).toContain(
    "--pr-monochrome-icon-filter: brightness(0)",
  );
  expect(list).toContain("启用");
  expect(list).toContain("禁用");
  expect(list).toContain("错误");
  expect(list).toContain("Badge, Button, Table, useNotification");
  expect(list).toContain('<Badge :variant="statusVariant(row.status)">');
  expect(list).not.toContain("<Tag");
  expect(list).toContain('class="agent-item-table"');
  expect(list).toContain("fixed-header");
  expect(list).toContain('layout="fixed"');
  expect(list.indexOf('key: "version"')).toBeLessThan(
    list.indexOf('key: "status"'),
  );
  expect(list.indexOf('key: "status"')).toBeLessThan(
    list.indexOf('key: "sourcePath"'),
  );
  expect(list).toContain("<template #cell-sourcePath");
  expect(list).toContain('icon="ph:copy"');
  expect(list).toContain("copySourcePath(row.sourcePath)");
  expect(list).toContain(
    'key: "actions",\n    title: "操作",\n    width: 64,\n    align: "right" as const,\n    fixed: "right" as const,',
  );
  expect(list).toContain('icon="ph:trash"');
  expect(list).toContain("emit('uninstall', row)");
  expect(list).toContain("text-overflow: ellipsis");
  expect(list).toContain("overflow-x: auto");
  expect(list).not.toContain("item-group-header");
  expect(list).not.toContain("@lobehub/icons-static-svg");
  expect(page).toContain(".item-results {\n  display: flex;");
  expect(page).toContain("overflow: hidden");
  expect(page).not.toContain("overflow: auto");
  expect(list).not.toContain("http");
});

test("智能体页的本地 command 不复用管理服务命令状态", () => {
  const page = source("pages/agents.vue");
  const workspace = source("composables/useAgentWorkspace.ts");
  const localCommand = source("composables/useLocalCommand.ts");
  const relayCommand = source("composables/useRelayCommand.ts");
  const nativeAgents = readFileSync(
    new URL("../src-tauri/src/agents.rs", import.meta.url),
    "utf8",
  );

  expect(page).toContain("useLocalCommand");
  expect(localCommand).toContain('from "@tauri-apps/api/core"');
  expect(localCommand).toContain('"agents_remove"');
  expect(localCommand).toContain('"agents_status"');
  expect(localCommand).toContain('"agent_items_get"');
  expect(relayCommand).not.toContain('"agents_list"');
  expect(workspace).toContain('"agents_status"');
  expect(workspace).toContain('"agent_items_get"');
  expect(workspace).not.toContain('"agents_versions"');
  expect(page).toContain("agentSettingsLoading");
  expect(workspace).toContain("Promise.all(");
  expect(page).not.toContain("isAgentContentLoading(client.client)");
  expect(page).toContain('v-if="!clientStatusesLoaded"');
  expect(page).toContain('text="正在检测智能体安装状态..."');
  expect(page).toContain('v-else-if="isAgentContentLoading(activeClient)"');
  expect(page).toContain('v-else-if="!isClientInstalled(activeClient)"');
  expect(page).toContain("未检测到本机安装");
  expect(nativeAgents).toContain("CREATE_NO_WINDOW");
  expect(nativeAgents).toContain("creation_flags(CREATE_NO_WINDOW.0)");
});
