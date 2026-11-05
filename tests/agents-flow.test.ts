import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("智能体工作区按客户端状态、内容与设置职责分层", () => {
  const app = source("app.vue");
  const page = source("pages/agents.vue");
  const workspace = source("composables/agent/workspace.ts");
  const sidebar = source("components/agents/workspace/Sidebar.vue");
  const clientIcon = source("components/agents/client/Icon.vue");
  const clientAvatarGroup = source("components/agents/client/AvatarGroup.vue");
  const content = source("components/agents/workspace/Content.vue");
  const extensionContent = source("components/extensions/library/Content.vue");
  const settings = source("composables/agent/settings.ts");
  const rules = source("composables/agent/rules.ts");

  expect(app).toContain("void agentWorkspace.refreshClientStatuses()");
  expect(page).toContain("useAgentWorkspace");
  expect(page).toContain("AgentSidebar");
  expect(page).toContain("AgentClientContent");
  expect(page).toContain("ExtensionLibraryContent");
  expect(page).toContain("AgentSettingsDrawer");
  expect(page).toContain("function refreshAgentClients()");
  expect(page).toContain('@click="refreshAgentClients"');
  expect(page).toContain("workspace.refreshClient(activeClient.value)");
  expect(page).not.toContain('"agents_list"');
  expect(workspace).toContain('"agents_status"');
  expect(workspace).toContain('"agent_items_get"');
  expect(workspace).toContain('"agent_settings_get"');
  expect(workspace).not.toContain('"agents_list"');
  expect(workspace).toContain("Promise.all(");

  expect(sidebar).toContain('class="agent-client-loading"');
  expect(sidebar).toContain("AgentClientIcon");
  expect(sidebar).not.toContain("agent-client-icon--uninstalled");
  expect(sidebar).not.toContain("agent-client-icon--monochrome");
  expect(sidebar).toContain("agent-sidebar-loading-spin");
  expect(sidebar).toContain('icon="ph:circle-notch" size="28"');
  expect(clientIcon).toContain("agentClientDefinitions");
  expect(clientIcon).toContain("var(--pr-monochrome-icon-filter)");
  expect(clientAvatarGroup).toContain("AvatarGroup");
  expect(clientAvatarGroup).toContain(':max="4"');
  expect(clientAvatarGroup).toContain('size="small"');
  expect(clientAvatarGroup).toContain(":deep(img)");
  expect(content).toContain('text="正在检测智能体安装状态..."');
  expect(content).toContain('text="正在读取智能体设置..."');
  expect(content).toContain("未检测到本机安装");
  expect(content).toContain("AgentRulesEditor");
  expect(content).toContain("AgentPluginList");
  expect(content).toContain("AgentMcpList");
  expect(content).toContain("AgentSkillList");
  expect(content).toContain("AgentHookList");
  expect(extensionContent).toContain("ExtensionCatalogTable");
  expect(content).toContain('icon="ph:sliders-horizontal"');
  expect(content).toContain('aria-label="配置"');

  expect(settings).toContain("codexConnection");
  expect(settings).not.toContain('kind: "custom"');
  expect(settings).not.toContain("customBaseUrl");
  expect(settings).not.toContain("customToken");
  expect(settings).not.toContain("openCode");
  expect(settings).not.toContain("modelName: group.name");
  expect(source("utils/agent/connections.ts")).toContain(
    "groupEndpointModels(endpoint.models)",
  );
  expect(settings).toContain(
    "description: `${groupEndpointModels(endpoint.models).length} 个模型`",
  );
  expect(settings).not.toContain(
    "endpoint.models.map(({ model_name, upstream_model })",
  );
  expect(settings).toContain("copyAgentClientSettings(configuration, draft");
  expect(rules).toContain(
    "function replace(client: AgentClient, rules: string)",
  );
  expect(rules).toContain("draft[client] = rules");
  expect(rules).toContain("saving.value = true");
  expect(rules).toContain("saving.value = false");
  expect(page).toContain("workspaceExit.register");
  expect(page).toContain("agentRules.saving.value");
});

test("智能体本地操作不复用管理服务命令状态", () => {
  const page = source("pages/agents.vue");
  const workspace = source("composables/agent/workspace.ts");
  const localCommand = source("composables/command/local.ts");
  const relayCommand = source("composables/command/relay.ts");
  const nativeAgents = readFileSync(
    new URL("../src-tauri/src/agents/discovery.rs", import.meta.url),
    "utf8",
  );

  expect(page).toContain("useLocalCommand");
  expect(localCommand).toContain('from "@tauri-apps/api/core"');
  expect(localCommand).toContain('"agents_remove"');
  expect(localCommand).toContain('"agents_status"');
  expect(localCommand).toContain('"agent_items_get"');
  expect(relayCommand).not.toContain('"agents_list"');
  expect(workspace).not.toContain('"agents_versions"');
  expect(nativeAgents).toContain("CREATE_NO_WINDOW");
  expect(nativeAgents).toContain("creation_flags(CREATE_NO_WINDOW.0)");
});

test("内置技能禁用删除操作", () => {
  const display = source("utils/agent/itemDisplay.ts");

  expect(display).toContain('item.source === "system"');
  expect(display).toContain(
    "item.package === null && Boolean(item.errorMessage)",
  );
  expect(display).toContain("内置技能不可卸载");
});

test("安装异常用标题后的状态图标和 tooltip 说明原因", () => {
  const error = source("components/agents/workspace/items/ItemError.vue");
  const sourceTag = source("components/agents/workspace/items/ItemSource.vue");

  expect(error).toContain("<Tooltip");
  expect(error).toContain(':content="message"');
  expect(error).toContain('icon="ph:warning-circle"');
  expect(error).toContain('role="img"');
  expect(error).toContain('tabindex="0"');
  expect(error).toContain(':aria-label="message"');
  expect(error).toContain("width: 14px");
  expect(error).toContain("height: 14px");
  expect(error).toContain("align-items: center");
  expect(error).not.toContain("<small");
  expect(sourceTag).toContain("agent-item-source-empty");
  expect(source("utils/agent/itemDisplay.ts")).toContain("状态错误不可卸载");
});

test("智能体条目按类型拆分展示，Hook 使用事件和匹配条件", () => {
  const content = source("components/agents/workspace/Content.vue");
  const plugin = source("components/agents/workspace/items/PluginList.vue");
  const mcp = source("components/agents/workspace/items/McpList.vue");
  const skill = source("components/agents/workspace/items/SkillList.vue");
  const hook = source("components/agents/workspace/items/HookList.vue");

  expect(content).toContain("activeSection === 'plugin'");
  expect(content).toContain("activeSection === 'mcp'");
  expect(content).toContain("activeSection === 'skill'");
  expect(content).toContain("AgentHookList");
  expect(plugin).not.toContain("<Table");
  expect(plugin).toContain("AgentItemRow");
  expect(mcp).not.toContain("<Table");
  expect(skill).not.toContain("<Table");
  expect(hook).not.toContain("<Table");
  expect(mcp).toContain("AgentItemRow");
  expect(skill).toContain("AgentItemRow");
  expect(hook).toContain("AgentItemRow");
  expect(plugin).toContain("<EmptyState");
  expect(mcp).toContain("<EmptyState");
  expect(skill).toContain("<EmptyState");
  expect(hook).toContain("<EmptyState");
  expect(hook).toContain("const groups = computed");
  expect(hook).toContain("group.event");
  expect(hook).toContain("hookMatcher(item) || '未限定'");
  expect(hook).toContain("item.command");
  expect(hook).not.toContain("hookHash");
});

test("智能体模型选项使用目录显示名且连接携带目录对象", () => {
  const settings = source("composables/agent/settings.ts");
  const agentUtils = source("utils/agent/settings.ts");
  const connections = source("utils/agent/connections.ts");

  expect(settings).toContain("label: model.display_name");
  expect(settings).toContain("catalogLanguageModel(group.catalogModel)");
  expect(settings).toContain("catalogModel");
  expect(connections).toContain("groupEndpointModels(endpoint.models)");
  expect(connections).toContain("models: endpointCatalogModels(endpoint)");
  expect(settings).toContain("codexConnectionFor(endpoint, relayUrl)");
  expect(settings).toContain("model.id");
  expect(settings).not.toContain("modelName:");
  expect(settings).not.toContain("upstreamModel:");
  expect(connections).toContain("catalogLanguageModel(group.catalogModel)");
  expect(agentUtils).toContain("CatalogLanguageModelResponse");
  expect(agentUtils).toContain("catalogModel?: CatalogLanguageModelResponse");
});

test("智能体设置表单定义了自己用到的分组与字段样式", () => {
  const contents = source("components/agents/settings/forms/Codex.vue");
  // 引用失效时接入点下拉框回落到占位文案，不额外加失效标记。
  expect(contents).toContain('label="接入点"');
  expect(contents).toContain('placeholder="选择接入点"');
  for (const rule of [
    "group",
    "group-header",
    "fields",
    "rows",
    "row",
    "label",
    "value",
  ]) {
    if (contents.includes(`class="agent-settings__${rule}"`)) {
      expect(contents).toContain(`.agent-settings__${rule}`);
    }
  }
});
