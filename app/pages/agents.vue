<script setup lang="ts">
import {
  Button,
  Drawer,
  Input,
  List,
  ListItem,
  MarkdownViewer,
  RadioGroup,
  Select,
  Textarea,
  Toggle,
  useNotification,
} from "@stellar/ui";
import type {
  AgentClient,
  AgentItemKind,
  AgentItemsSnapshot,
  AgentSettingsSnapshot,
  BootstrapState,
  RelayEndpoint,
} from "~/stores/relay";
import { useRelayStore } from "~/stores/relay";
import codexIcon from "@lobehub/icons-static-svg/icons/openai.svg";
import claudeIcon from "@lobehub/icons-static-svg/icons/claudecode-color.svg";
import AgentItemList from "~/components/agents/AgentItemList.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

type AgentSection = "rules" | AgentItemKind;

const customEndpointValue = "__custom__";

const { pending: agentsPending, invokeLocalCommand } = useLocalCommand();
const { pending: endpointsPending, invokeCommand } = useRelayCommand();
const notifications = useNotification();
const workspaceExit = useWorkspaceExitGuard();
const { bootstrap, setBootstrap } = useRelayStore();
const snapshot = ref<AgentItemsSnapshot>({ clients: [] });
const endpoints = ref<RelayEndpoint[]>([]);
const activeClient = ref<AgentClient>("codex");
const activeSection = ref<AgentSection>("rules");
const rulesEditorElement = ref<HTMLElement | null>(null);
const rulesPreviewElement = ref<HTMLElement | null>(null);
let rulesEditorTextarea: HTMLTextAreaElement | null = null;
let rulesScrollSyncing = false;
let rulesSaveTimer: ReturnType<typeof setTimeout> | undefined;
let rulesLoaded = false;
let suppressRulesSave = false;
const showSettings = ref(false);
const agentConfiguration = reactive(createAgentConfiguration());
const settingsDraft = reactive(createAgentConfiguration());
const rulesDraft = reactive({ codex: "", claudeCode: "" });
let settingsExitRegistration:
  | ReturnType<typeof workspaceExit.register>
  | undefined;
let rulesExitRegistration: ReturnType<typeof workspaceExit.register> | undefined;

function createAgentConfiguration() {
  return {
  codex: {
    endpoint: "",
    customBaseUrl: "",
    customToken: "",
    model: "",
    reasoningEffort: "high",
    personality: "pragmatic",
    webSearch: true,
    sandbox: "workspace-write",
    disableResponseStorage: true,
    maxThreads: 16,
    maxDepth: 1,
    jobMaxRuntimeSeconds: 1800,
    networkAccess: true,
    shellEnvironmentInherit: "all",
    windowsSandbox: "unelevated",
    features: {
      memories: true,
      goals: true,
      workspaceDependencies: false,
    },
    rules: "",
  },
  claudeCode: {
    endpoint: "",
    opusModel: "",
    sonnetModel: "",
    haikuModel: "",
    subagentModel: "",
    effort: "high",
    language: "中文",
    permissionMode: "acceptEdits",
    rules: "",
  },
  };
}

const clients: Array<{ client: AgentClient; label: string; icon: string }> = [
  { client: "codex", label: "Codex", icon: codexIcon },
  { client: "claudeCode", label: "Claude Code", icon: claudeIcon },
];
const sectionOptions = [
  { value: "rules", label: "规则", icon: "ph:notebook" },
  { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
  { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
  { value: "skill", label: "Skill", icon: "ph:book-open-text" },
];
const endpointOptions = computed(() =>
  [
    ...endpoints.value.map((endpoint) => ({
      value: endpoint.id,
      label: endpoint.name,
      description: `${endpoint.models.length} 个模型`,
    })),
    { value: customEndpointValue, label: "自定义" },
  ],
);
const activeSettings = computed(() => settingsDraft[activeClient.value]);
const selectedEndpoint = computed(() =>
  endpoints.value.find((endpoint) => endpoint.id === activeSettings.value.endpoint),
);
const isCustomCodexEndpoint = computed(
  () => settingsDraft.codex.endpoint === customEndpointValue,
);
const modelOptions = computed(() =>
  (selectedEndpoint.value?.models ?? []).map((model) => ({
    value: model.model_name,
    label: model.model_name,
  })),
);
const activeItems = computed(
  () =>
    snapshot.value.clients.find((client) => client.client === activeClient.value)
      ?.items ?? [],
);
const activeKind = computed<AgentItemKind | null>(() =>
  activeSection.value === "rules" ? null : activeSection.value,
);
const sectionItems = computed(() =>
  activeKind.value
    ? activeItems.value.filter((item) => item.kind === activeKind.value)
    : [],
);
const pending = computed(() => agentsPending.value || endpointsPending.value);
const settingsDirty = computed(
  () => JSON.stringify(settingsDraft) !== JSON.stringify(agentConfiguration),
);
const rulesDirty = computed(
  () =>
    rulesDraft.codex !== agentConfiguration.codex.rules ||
    rulesDraft.claudeCode !== agentConfiguration.claudeCode.rules,
);
const activeRules = computed(() => rulesDraft[activeClient.value]);

const effortOptions = [
  { value: "low", label: "低" },
  { value: "medium", label: "中" },
  { value: "high", label: "高" },
  { value: "xhigh", label: "很高" },
];
const personalityOptions = [
  { value: "pragmatic", label: "务实" },
  { value: "friendly", label: "友好" },
  { value: "direct", label: "直接" },
];
const sandboxOptions = [
  { value: "read-only", label: "只读" },
  { value: "workspace-write", label: "工作区写入" },
  { value: "danger-full-access", label: "完全访问" },
];
const permissionOptions = [
  { value: "manual", label: "手动确认" },
  { value: "acceptEdits", label: "允许编辑" },
  { value: "auto", label: "自动执行" },
];
const languageOptions = [
  { value: "中文", label: "中文" },
  { value: "English", label: "English" },
];
const shellEnvironmentOptions = [
  { value: "all", label: "全部继承" },
  { value: "core", label: "仅基础环境" },
  { value: "none", label: "不继承" },
];
const windowsSandboxOptions = [
  { value: "unelevated", label: "非提升" },
  { value: "elevated", label: "提升" },
];
const codexFeatures = [
  { key: "memories", label: "长期记忆" },
  { key: "goals", label: "目标管理" },
  { key: "workspaceDependencies", label: "工作区依赖" },
] as const;

function updateCodexNumber(
  key: "maxThreads" | "maxDepth" | "jobMaxRuntimeSeconds",
  value: string | number,
) {
  settingsDraft.codex[key] = Number(value);
}

function syncRulesScroll(source: HTMLElement, target: HTMLElement) {
  if (rulesScrollSyncing) return;
  const sourceRange = source.scrollHeight - source.clientHeight;
  const targetRange = target.scrollHeight - target.clientHeight;
  if (sourceRange <= 0 || targetRange <= 0) return;

  rulesScrollSyncing = true;
  target.scrollTop = (source.scrollTop / sourceRange) * targetRange;
  requestAnimationFrame(() => {
    rulesScrollSyncing = false;
  });
}

function onRulesEditorScroll() {
  if (rulesEditorTextarea && rulesPreviewElement.value) {
    syncRulesScroll(rulesEditorTextarea, rulesPreviewElement.value);
  }
}

function onRulesPreviewScroll() {
  if (rulesPreviewElement.value && rulesEditorTextarea) {
    syncRulesScroll(rulesPreviewElement.value, rulesEditorTextarea);
  }
}

function unbindRulesScroll() {
  rulesEditorTextarea?.removeEventListener("scroll", onRulesEditorScroll);
  rulesPreviewElement.value?.removeEventListener("scroll", onRulesPreviewScroll);
  rulesEditorTextarea = null;
}

function bindRulesScroll() {
  unbindRulesScroll();
  rulesEditorTextarea = rulesEditorElement.value?.querySelector("textarea") ?? null;
  rulesEditorTextarea?.addEventListener("scroll", onRulesEditorScroll, { passive: true });
  rulesPreviewElement.value?.addEventListener("scroll", onRulesPreviewScroll, {
    passive: true,
  });
}

function copyClientSettings(
  source: ReturnType<typeof createAgentConfiguration>,
  target: ReturnType<typeof createAgentConfiguration>,
  client: AgentClient,
) {
  if (client === "codex") {
    Object.assign(target.codex, source.codex, {
      features: { ...source.codex.features },
    });
    return;
  }
  Object.assign(target.claudeCode, source.claudeCode);
}

function settingsPayload(
  codex = agentConfiguration.codex,
  claudeCode = agentConfiguration.claudeCode,
) {
  const { customToken, ...codexSettings } = codex;
  return {
    codex: { ...codexSettings, features: { ...codexSettings.features } },
    claudeCode: { ...claudeCode },
  };
}

function codexConnection() {
  const customBaseUrl = settingsDraft.codex.customBaseUrl.trim();
  if (isCustomCodexEndpoint.value && customBaseUrl) {
    return {
      kind: "custom",
      baseUrl: customBaseUrl,
      token: settingsDraft.codex.customToken,
    };
  }
  const endpoint = selectedEndpoint.value;
  if (endpoint && bootstrap.value?.relay_url) {
    return {
      kind: "prelay",
      endpointId: endpoint.id,
      endpointName: endpoint.name,
      endpointToken: endpoint.token,
      relayUrl: bootstrap.value.relay_url,
    };
  }
  return null;
}

function claudeCodeConnection() {
  const endpoint = selectedEndpoint.value;
  if (endpoint && bootstrap.value?.relay_url) {
    return {
      kind: "prelay",
      endpointToken: endpoint.token,
      relayUrl: bootstrap.value.relay_url,
    };
  }
  return null;
}

function openSettings() {
  copyClientSettings(agentConfiguration, settingsDraft, "codex");
  copyClientSettings(agentConfiguration, settingsDraft, "claudeCode");
  showSettings.value = true;
}

async function selectClient(client: AgentClient) {
  if (showSettings.value && settingsExitRegistration) {
    const canSwitch = await settingsExitRegistration.requestExit();
    if (!canSwitch) return;
  }
  activeClient.value = client;
}

function closeSettingsImmediately() {
  showSettings.value = false;
}

function discardSettingsDraft() {
  copyClientSettings(agentConfiguration, settingsDraft, "codex");
  copyClientSettings(agentConfiguration, settingsDraft, "claudeCode");
  closeSettingsImmediately();
}

function requestCloseSettings() {
  if (settingsExitRegistration) {
    void settingsExitRegistration.requestExit();
    return;
  }
  closeSettingsImmediately();
}

function updateSettingsVisibility(visible: boolean) {
  if (visible) showSettings.value = true;
  else requestCloseSettings();
}

async function saveSettings() {
  const client = activeClient.value;
  try {
    await invokeLocalCommand("agent_settings_save", {
      client,
      settings:
        client === "codex"
          ? settingsPayload(settingsDraft.codex)
          : settingsPayload(agentConfiguration.codex, settingsDraft.claudeCode),
       codexConnection: client === "codex" ? codexConnection() : null,
       claudeCodeConnection:
         client === "claudeCode" ? claudeCodeConnection() : null,
    });
    copyClientSettings(settingsDraft, agentConfiguration, client);
    showSettings.value = false;
    notifications.success("设置已保存");
  } catch {
    // The local command composable exposes the stable error to this view.
  }
}

function scheduleRulesSave(client: AgentClient) {
  if (!rulesLoaded || suppressRulesSave) return;
  if (rulesSaveTimer) clearTimeout(rulesSaveTimer);
  rulesSaveTimer = setTimeout(() => {
    void saveRules(client);
  }, 500);
}

function discardRulesDraft() {
  if (rulesSaveTimer) clearTimeout(rulesSaveTimer);
  suppressRulesSave = true;
  rulesDraft.codex = agentConfiguration.codex.rules;
  rulesDraft.claudeCode = agentConfiguration.claudeCode.rules;
  queueMicrotask(() => {
    suppressRulesSave = false;
  });
}

async function saveRules(client: AgentClient) {
  const codex = {
    ...agentConfiguration.codex,
    rules: client === "codex" ? rulesDraft.codex : agentConfiguration.codex.rules,
  };
  const claudeCode = {
    ...agentConfiguration.claudeCode,
    rules:
      client === "claudeCode"
        ? rulesDraft.claudeCode
        : agentConfiguration.claudeCode.rules,
  };
  try {
    await invokeLocalCommand("agent_settings_save", {
      client,
      settings: settingsPayload(codex, claudeCode),
       codexConnection: null,
       claudeCodeConnection: null,
    });
    agentConfiguration[client].rules = rulesDraft[client];
    notifications.success("规则已保存");
  } catch {
    // The local command composable exposes the stable error to this view.
  }
}

function normalizeBaseUrl(url: string) {
  return url.trim().replace(/\/+$/, "");
}

function managementBaseUrl(relayUrl: string) {
  const normalized = normalizeBaseUrl(relayUrl);
  return normalized.endsWith("/v1") ? normalized : `${normalized}/v1`;
}

async function loadAgentPage() {
  rulesLoaded = false;
  try {
    snapshot.value = await invokeLocalCommand<AgentItemsSnapshot>(
      "agents_list",
    );
  } catch {
    // The local command composable exposes the stable error to this view.
  }
  try {
    endpoints.value = await invokeCommand<RelayEndpoint[]>("endpoints_list");
  } catch {
    // The application-level management API status owns endpoint failures.
  }
  if (!bootstrap.value) {
    try {
      setBootstrap(await invokeCommand<BootstrapState>("bootstrap"));
    } catch {
      // The application-level management API status owns bootstrap failures.
    }
  }
  try {
    const settings = await invokeLocalCommand<AgentSettingsSnapshot>(
      "agent_settings_get",
    );
    const { endpointName, baseUrl, customToken, ...codexSettings } = settings.codex;
    Object.assign(agentConfiguration.codex, codexSettings);
    Object.assign(agentConfiguration.codex.features, settings.codex.features);
    const { baseUrl: claudeBaseUrl, endpointToken, ...claudeCodeSettings } =
      settings.claudeCode;
    Object.assign(agentConfiguration.claudeCode, claudeCodeSettings);
    agentConfiguration.codex.customBaseUrl = baseUrl ?? "";
    const managementUrl = bootstrap.value?.relay_url
      ? managementBaseUrl(bootstrap.value.relay_url)
      : null;
    const codexEndpoint =
      managementUrl && baseUrl && normalizeBaseUrl(baseUrl) === managementUrl
        ? endpoints.value.find((endpoint) => endpoint.name === endpointName)
        : undefined;
    agentConfiguration.codex.endpoint = codexEndpoint?.id ?? customEndpointValue;
    agentConfiguration.codex.customToken = codexEndpoint ? "" : customToken ?? "";
    const claudeEndpoint =
      managementUrl &&
      claudeBaseUrl &&
      normalizeBaseUrl(claudeBaseUrl) === managementUrl
        ? endpoints.value.find((endpoint) => endpoint.token === endpointToken)
        : undefined;
    agentConfiguration.claudeCode.endpoint =
      claudeEndpoint?.id ?? customEndpointValue;
    copyClientSettings(agentConfiguration, settingsDraft, "codex");
    copyClientSettings(agentConfiguration, settingsDraft, "claudeCode");
    rulesDraft.codex = agentConfiguration.codex.rules;
    rulesDraft.claudeCode = agentConfiguration.claudeCode.rules;
    await nextTick();
    rulesLoaded = true;
  } catch {
    // The local command composable exposes the stable error to this view.
  }
}

onMounted(() => {
  rulesExitRegistration = workspaceExit.register({
    close: discardRulesDraft,
    state: () =>
      agentsPending.value ? "blocked" : rulesDirty.value ? "discard" : "allow",
  });
  void loadAgentPage();
});

watch(showSettings, (visible) => {
  if (!visible) {
    settingsExitRegistration?.unregister();
    settingsExitRegistration = undefined;
    return;
  }
  settingsExitRegistration = workspaceExit.register({
    close: discardSettingsDraft,
    state: () =>
      pending.value ? "blocked" : settingsDirty.value ? "discard" : "allow",
  });
});

watch(
  () => settingsDraft.codex.endpoint,
  (endpoint, previous) => {
    if (
      showSettings.value &&
      endpoint === customEndpointValue &&
      previous !== customEndpointValue
    ) {
      settingsDraft.codex.customBaseUrl = "";
      settingsDraft.codex.customToken = "";
    }
  },
);

watch(
  () => rulesDraft.codex,
  () => scheduleRulesSave("codex"),
);

watch(
  () => rulesDraft.claudeCode,
  () => scheduleRulesSave("claudeCode"),
);

watch(activeSection, async (section) => {
  unbindRulesScroll();
  if (section !== "rules") return;
  await nextTick();
  bindRulesScroll();
});

onBeforeUnmount(() => {
  if (rulesSaveTimer) clearTimeout(rulesSaveTimer);
  settingsExitRegistration?.unregister();
  rulesExitRegistration?.unregister();
  unbindRulesScroll();
});
</script>

<template>
  <main class="page-dashboard">
    <PanelSection title="智能体">
      <template #header-actions>
        <Button
          variant="primary"
          icon="ph:arrows-clockwise"
          :loading="pending"
          aria-label="刷新"
          title="刷新"
          @click="loadAgentPage"
        >
          刷新
        </Button>
      </template>
      <div class="agent-content">
        <List class="agent-client-list" :divided="false">
          <ListItem
            v-for="client in clients"
            :key="client.client"
            :active="activeClient === client.client"
            clickable
            @click="selectClient(client.client)"
          >
            <template #prefix>
              <img
                :src="client.icon"
                :alt="client.label"
                class="agent-client-icon"
                :class="{
                  'agent-client-icon--monochrome': client.client === 'codex',
                }"
              />
            </template>
            {{ client.label }}
          </ListItem>
        </List>
        <div class="agent-main">
          <div class="agent-toolbar">
            <RadioGroup
              v-model="activeSection"
              :options="sectionOptions"
              variant="button"
            />
            <div class="agent-toolbar__actions">
              <Button
                square
                icon="ph:gear-six"
                aria-label="编辑设置"
                title="编辑设置"
                @click="openSettings"
              />
            </div>
          </div>
          <section v-if="activeSection === 'rules'" class="agent-rules">
            <div ref="rulesEditorElement" class="agent-rules__editor">
              <Textarea
                v-model="rulesDraft[activeClient]"
                class="agent-rules__input"
                aria-label="编辑全局规则"
                :rows="18"
                resize="none"
              />
            </div>
            <div ref="rulesPreviewElement" class="agent-rules__preview">
              <MarkdownViewer
                :content="activeRules"
                class="agent-settings__markdown"
              />
            </div>
          </section>
          <div v-else class="item-results">
            <AgentItemList :items="sectionItems" />
          </div>
        </div>
      </div>
    </PanelSection>
  </main>
  <Drawer
    :visible="showSettings"
    title="智能体设置"
    size="xlarge"
    :blocked="pending || settingsDirty"
    @update:visible="updateSettingsVisibility"
  >
    <form id="agent-settings-form" class="agent-settings-form" @submit.prevent="saveSettings">
      <template v-if="activeClient === 'codex'">
          <section class="agent-settings__group">
            <div class="agent-settings__group-header">
              <h3>模型与接入</h3>
            </div>
            <div class="agent-settings__fields">
              <Select
                v-model="settingsDraft.codex.endpoint"
                label="接入点"
                :options="endpointOptions"
              />
              <template v-if="isCustomCodexEndpoint">
                <Input v-model="settingsDraft.codex.model" label="默认模型" />
                <Input
                  v-model="settingsDraft.codex.customBaseUrl"
                  label="Base URL"
                  placeholder="https://api.example.com/v1"
                />
                <Input
                  v-model="settingsDraft.codex.customToken"
                  label="Token"
                />
              </template>
              <Select
                v-else
                v-model="settingsDraft.codex.model"
                label="默认模型"
                :options="modelOptions"
              />
            </div>
            <div class="agent-settings__rows">
              <div class="agent-settings__row">
                <span class="agent-settings__label">推理强度</span>
                <RadioGroup
                  v-model="settingsDraft.codex.reasoningEffort"
                  :options="effortOptions"
                  size="small"
                  variant="button"
                />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">交互风格</span>
                <RadioGroup
                  v-model="settingsDraft.codex.personality"
                  :options="personalityOptions"
                  size="small"
                  variant="button"
                />
              </div>
            </div>
          </section>

          <section class="agent-settings__group">
            <div class="agent-settings__group-header">
              <h3>执行与网络</h3>
            </div>
            <div class="agent-settings__rows">
              <div class="agent-settings__row">
                <span class="agent-settings__label">执行权限</span>
                <RadioGroup
                  v-model="settingsDraft.codex.sandbox"
                  :options="sandboxOptions"
                  size="small"
                  variant="button"
                />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">实时网络搜索</span>
                <Toggle v-model="settingsDraft.codex.webSearch" aria-label="实时网络搜索" />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">工作区网络访问</span>
                <Toggle v-model="settingsDraft.codex.networkAccess" aria-label="工作区网络访问" />
              </div>
            </div>
          </section>

          <section class="agent-settings__group">
            <div class="agent-settings__group-header">
              <h3>协作与记忆</h3>
            </div>
            <div class="agent-settings__rows">
              <div class="agent-settings__row">
                <span class="agent-settings__label">最大并发智能体</span>
                <Input
                  :model-value="settingsDraft.codex.maxThreads"
                  class="agent-settings__value"
                  type="number"
                  :min="1"
                  aria-label="最大并发智能体"
                  @update:model-value="updateCodexNumber('maxThreads', $event)"
                />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">子智能体最大深度</span>
                <Input
                  :model-value="settingsDraft.codex.maxDepth"
                  class="agent-settings__value"
                  type="number"
                  :min="0"
                  aria-label="子智能体最大深度"
                  @update:model-value="updateCodexNumber('maxDepth', $event)"
                />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">单项任务时限</span>
                <Input
                  :model-value="settingsDraft.codex.jobMaxRuntimeSeconds"
                  class="agent-settings__value"
                  type="number"
                  :min="1"
                  aria-label="单项任务时限（秒）"
                  @update:model-value="updateCodexNumber('jobMaxRuntimeSeconds', $event)"
                />
              </div>
              <div
                v-for="feature in codexFeatures"
                :key="feature.key"
                class="agent-settings__row"
              >
                <span class="agent-settings__label">{{ feature.label }}</span>
                <Toggle
                  v-model="settingsDraft.codex.features[feature.key]"
                  :aria-label="feature.label"
                />
              </div>
            </div>
          </section>

          <section class="agent-settings__group">
            <div class="agent-settings__group-header">
              <h3>运行环境</h3>
            </div>
            <div class="agent-settings__rows">
              <div class="agent-settings__row">
                <span class="agent-settings__label">禁用响应存储</span>
                <Toggle
                  v-model="settingsDraft.codex.disableResponseStorage"
                  aria-label="禁用响应存储"
                />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">Shell 环境继承</span>
                <Select
                  v-model="settingsDraft.codex.shellEnvironmentInherit"
                  class="agent-settings__value"
                  aria-label="Shell 环境继承"
                  :options="shellEnvironmentOptions"
                />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">Windows 沙箱</span>
                <RadioGroup
                  v-model="settingsDraft.codex.windowsSandbox"
                  :options="windowsSandboxOptions"
                  size="small"
                  variant="button"
                />
              </div>
            </div>
          </section>
      </template>

      <template v-else>
          <section class="agent-settings__group">
            <div class="agent-settings__group-header">
              <h3>模型与接入</h3>
            </div>
            <div class="agent-settings__fields">
              <Select
                v-model="settingsDraft.claudeCode.endpoint"
                label="接入点"
                :options="endpointOptions"
              />
              <Select
                v-model="settingsDraft.claudeCode.language"
                label="界面语言"
                :options="languageOptions"
              />
              <Select
                v-model="settingsDraft.claudeCode.opusModel"
                label="Opus 模型"
                :options="modelOptions"
              />
              <Select
                v-model="settingsDraft.claudeCode.sonnetModel"
                label="Sonnet 模型"
                :options="modelOptions"
              />
              <Select
                v-model="settingsDraft.claudeCode.haikuModel"
                label="Haiku 模型"
                :options="modelOptions"
              />
              <Select
                v-model="settingsDraft.claudeCode.subagentModel"
                label="子智能体模型"
                :options="modelOptions"
              />
            </div>
          </section>

          <section class="agent-settings__group">
            <div class="agent-settings__group-header">
              <h3>执行与协作</h3>
            </div>
            <div class="agent-settings__rows">
              <div class="agent-settings__row">
                <span class="agent-settings__label">推理强度</span>
                <RadioGroup
                  v-model="settingsDraft.claudeCode.effort"
                  :options="effortOptions"
                  size="small"
                  variant="button"
                />
              </div>
              <div class="agent-settings__row">
                <span class="agent-settings__label">工具权限</span>
                <RadioGroup
                  v-model="settingsDraft.claudeCode.permissionMode"
                  :options="permissionOptions"
                  size="small"
                  variant="button"
                />
              </div>
            </div>
          </section>
      </template>
    </form>
    <template #footer>
      <Button :disabled="pending" @click="requestCloseSettings">取消</Button>
      <Button
        form="agent-settings-form"
        type="submit"
        variant="primary"
        :disabled="pending"
      >
        {{ pending ? "保存中..." : "保存" }}
      </Button>
    </template>
  </Drawer>
</template>

<style scoped>
.page-dashboard {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--pr-dashboard-padding);
}

.agent-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 0 0 var(--spacing-md);
}

.agent-content {
  display: grid;
  flex: 1;
  min-height: 0;
  grid-template-columns: 184px minmax(0, 1fr);
}

.agent-client-list {
  min-height: 0;
  padding-right: var(--spacing-md);
  border-right: 1px solid var(--st-border-divider);
}

.agent-client-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.agent-client-icon--monochrome {
  filter: var(--pr-monochrome-icon-filter);
}

.agent-main {
  display: flex;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  padding-left: var(--spacing-lg);
}

.item-results {
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
  padding-top: var(--spacing-md);
  overflow: hidden;
}

.agent-toolbar__actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.agent-settings-form {
  display: grid;
  gap: var(--spacing-xl);
  padding: var(--spacing-lg);
}

.agent-rules {
  display: grid;
  flex: 1;
  width: 100%;
  min-width: 0;
  min-height: 0;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  grid-template-rows: minmax(0, 1fr);
  gap: var(--spacing-xl);
  overflow: hidden;
  padding: 0 0 var(--spacing-lg);
}

.agent-rules__editor,
.agent-rules__preview {
  min-width: 0;
  min-height: 0;
}

.agent-rules__editor {
  display: grid;
  min-height: 0;
  grid-template-rows: minmax(0, 1fr);
  align-content: start;
  gap: var(--spacing-sm);
}

.agent-rules__preview {
  overflow-y: auto;
}

.agent-rules__input {
  height: 100%;
  min-height: 0;
}

.agent-rules__input :deep(textarea) {
  height: 100%;
}

.agent-settings__group {
  display: grid;
  min-width: 0;
  gap: var(--spacing-lg);
}

.agent-settings__group-header h3 {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 15px;
  font-weight: 600;
}

.agent-settings__group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.agent-settings__fields {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-lg);
}

.agent-settings__rows {
  display: grid;
  gap: var(--spacing-lg);
}

.agent-settings__row {
  display: grid;
  min-height: 36px;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--spacing-xl);
}

.agent-settings__label {
  color: var(--st-text-secondary);
  font-size: 14px;
  font-weight: 400;
}

.agent-settings__value {
  width: 168px;
}

.agent-settings__markdown {
  min-width: 0;
}

</style>
