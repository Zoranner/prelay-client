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
  const workspace = source("composables/useAgentWorkspace.ts");
  const sidebar = source("components/agents/AgentSidebar.vue");
  const content = source("components/agents/AgentWorkspaceContent.vue");
  const settings = source("composables/useAgentSettings.ts");
  const rules = source("composables/useAgentRules.ts");

  expect(app).toContain("void agentWorkspace.refreshClientStatuses()");
  expect(page).toContain("useAgentWorkspace");
  expect(page).toContain("AgentSidebar");
  expect(page).toContain("AgentWorkspaceContent");
  expect(page).toContain("AgentSettingsDrawer");
  expect(page).toContain("function refreshAgentClients()");
  expect(page).toContain('@click="refreshAgentClients"');
  expect(page).toContain("agentWorkspace.refreshClient(activeClient.value)");
  expect(page).not.toContain('"agents_list"');
  expect(page).not.toContain("claudeCode");
  expect(page).not.toContain("ClaudeCode");
  expect(workspace).toContain('"agents_status"');
  expect(workspace).toContain('"agent_items_get"');
  expect(workspace).toContain('"agent_settings_get"');
  expect(workspace).not.toContain('"agents_list"');
  expect(workspace).toContain("Promise.all(");

  expect(sidebar).toContain('class="agent-client-loading"');
  expect(sidebar).toContain("agent-client-icon--uninstalled");
  expect(sidebar).toContain("agent-client-icon--monochrome");
  expect(sidebar).toContain("agent-sidebar-loading-spin");
  expect(sidebar).toContain('icon="ph:circle-notch" size="28"');
  expect(content).toContain('text="正在检测智能体安装状态..."');
  expect(content).toContain('text="正在读取智能体设置..."');
  expect(content).toContain("未检测到本机安装");
  expect(content).toContain("AgentRulesEditor");
  expect(content).toContain("AgentItemList");
  expect(content).toContain("ExtensionCatalogTable");
  expect(content).toContain('icon="ph:sliders-horizontal"');
  expect(content).toContain('aria-label="配置"');

  expect(settings).toContain("codexConnection");
  expect(settings).toContain("openCodeConnection");
  expect(settings).toContain("endpointToken");
  expect(settings).toContain("groupEndpointModels(endpoint.models)");
  expect(settings).toContain("modelName: group.name");
  expect(settings).toContain(
    "description: `${groupEndpointModels(endpoint.models).length} 个模型`",
  );
  expect(settings).not.toContain(
    "endpoint.models.map(({ model_name, upstream_model })",
  );
  expect(settings).toContain("copyAgentClientSettings(configuration, draft");
  expect(rules).toContain(
    'const draft = reactive({ codexCli: "", chatgpt: "", openCode: "" })',
  );
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
  const workspace = source("composables/useAgentWorkspace.ts");
  const localCommand = source("composables/useLocalCommand.ts");
  const relayCommand = source("composables/useRelayCommand.ts");
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

test("智能体模型选项使用目录显示名且连接携带目录对象", () => {
  const settings = source("composables/useAgentSettings.ts");
  const agentUtils = source("utils/agentSettings.ts");
  const codex = readFileSync(
    new URL("../src-tauri/src/agents/settings/codex.rs", import.meta.url),
    "utf8",
  );
  const settingsMod = readFileSync(
    new URL("../src-tauri/src/agents/settings/mod.rs", import.meta.url),
    "utf8",
  );

  expect(settings).toContain("label: group.displayName");
  expect(settings).toContain("catalogLanguageModel(group.catalogModel)");
  expect(settings).toContain("catalogModel");
  expect(settings).toContain("modelName: group.name");
  expect(settings).toContain("upstreamModel:");
  expect(agentUtils).toContain("CatalogLanguageModelResponse");
  expect(codex).not.toContain("catalog_model");
  expect(codex).toContain('profile["slug"] = Value::String(model.id.clone())');
  expect(settingsMod).toContain(
    "models: Vec<CatalogLanguageModelResponse>",
  );
});
