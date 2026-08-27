<script setup lang="ts">
import {
  Button,
  Drawer,
  Icon,
  List,
  ListItem,
  Loading,
  RadioGroup,
  useConfirm,
  useNotification,
} from "@stellar/ui";
import type {
  AgentClient,
  AgentItem,
  AgentItemKind,
  AgentSettings,
  BootstrapState,
  ExtensionKind,
  ExtensionPackage,
  RelayEndpoint,
} from "~/stores/relay";
import { agentClientDefinitions, clientSupportsSettings } from "~/utils/agentClient";
import { createAgentConfiguration } from "~/utils/agentSettings";
import { useRelayStore } from "~/stores/relay";
import AgentItemList from "~/components/agents/AgentItemList.vue";
import AgentRulesEditor from "~/components/agents/AgentRulesEditor.vue";
import ClaudeCodeSettingsForm from "~/components/agents/ClaudeCodeSettingsForm.vue";
import ChatGptSettingsForm from "~/components/agents/ChatGptSettingsForm.vue";
import CodexSettingsForm from "~/components/agents/CodexSettingsForm.vue";
import ExtensionCatalogTable from "~/components/extensions/ExtensionCatalogTable.vue";
import ExtensionDetailDrawer from "~/components/extensions/ExtensionDetailDrawer.vue";
import ExtensionInstallModal from "~/components/extensions/ExtensionInstallModal.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

type AgentSection = "rules" | AgentItemKind;

const customEndpointValue = "__custom__";

const { pending: agentsPending, invokeLocalCommand } = useLocalCommand();
const { pending: endpointsPending, invokeCommand } = useRelayCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
const workspaceExit = useWorkspaceExitGuard();
const { bootstrap, setBootstrap } = useRelayStore();
const agentWorkspace = useAgentWorkspace();
const extensionCatalog = useExtensionCatalog();
const {
  snapshot,
  loaded: agentsLoaded,
  settings: cachedAgentSettings,
  settingsLoading: agentSettingsLoading,
  settingsLoaded: agentSettingsLoaded,
} = agentWorkspace;
const endpoints = ref<RelayEndpoint[]>([]);
const activeClient = ref<AgentClient>("codexCli");
const activeSection = ref<AgentSection>("rules");
const activeExtensionSection = ref<ExtensionKind>("rule");
const showExtensionCatalog = ref(false);
const selectedExtension = ref<ExtensionPackage | null>(null);
const showExtensionDetails = ref(false);
const showExtensionInstall = ref(false);
let rulesSaveTimer: ReturnType<typeof setTimeout> | undefined;
let rulesLoaded = false;
const rulesSaving = ref(false);
let suppressRulesSave = false;
const showSettings = ref(false);
const agentConfiguration = reactive(createAgentConfiguration());
const settingsDraft = reactive(createAgentConfiguration());
const rulesDraft = reactive({ codexCli: "", chatgpt: "", claudeCode: "" });
let settingsExitRegistration:
  ReturnType<typeof workspaceExit.register> | undefined;
let rulesExitRegistration:
  ReturnType<typeof workspaceExit.register> | undefined;

const agentClients = computed(() =>
  agentClientDefinitions.map((definition) => {
    const detected = snapshot.value.clients.find(
      ({ client }) => client === definition.client,
    );
    return {
      ...definition,
      installed: Boolean(detected),
      version: detected?.version ?? "-",
    };
  }),
);
const sectionOptions = [
  { value: "rules", label: "规则", icon: "ph:notebook" },
  { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
  { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
  { value: "skill", label: "Skill", icon: "ph:book-open-text" },
];
const extensionSectionOptions = [
  { value: "rule", label: "规则", icon: "ph:notebook" },
  { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
  { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
  { value: "skill", label: "Skill", icon: "ph:book-open-text" },
];
const endpointOptions = computed(() => [
  ...endpoints.value.map((endpoint) => ({
    value: endpoint.id,
    label: endpoint.name,
    description: `${endpoint.models.length} 个模型`,
  })),
  { value: customEndpointValue, label: "自定义" },
]);
const activeSettings = computed(() => settingsDraft[activeClient.value]);
const selectedEndpoint = computed(() =>
  endpoints.value.find(
    (endpoint) => endpoint.id === activeSettings.value.endpoint,
  ),
);
const isCustomCodexEndpoint = computed(
  () => activeSettings.value.endpoint === customEndpointValue,
);
const modelOptions = computed(() =>
  (selectedEndpoint.value?.models ?? []).map((model) => ({
    value: model.model_name,
    label: model.model_name,
  })),
);
const activeItems = computed(
  () =>
    snapshot.value.clients.find(
      (client) => client.client === activeClient.value,
    )?.items ?? [],
);
const activeClientDetected = computed(() =>
  isClientInstalled(activeClient.value),
);
const activeKind = computed<AgentItemKind | null>(() =>
  activeSection.value === "rules" ? null : activeSection.value,
);
const sectionItems = computed(() =>
  activeKind.value
    ? activeItems.value.filter((item) => item.kind === activeKind.value)
    : [],
);
const extensionPackages = computed(() =>
  extensionCatalog.catalog.value.packages.filter(
    (item) => item.kind === activeExtensionSection.value,
  ),
);
const pending = computed(() => agentsPending.value || endpointsPending.value);
const settingsDirty = computed(
  () => JSON.stringify(settingsDraft) !== JSON.stringify(agentConfiguration),
);
const rulesDirty = computed(
  () =>
    rulesDraft.codexCli !== agentConfiguration.codexCli.rules ||
    rulesDraft.chatgpt !== agentConfiguration.chatgpt.rules ||
    rulesDraft.claudeCode !== agentConfiguration.claudeCode.rules,
);

function isClientInstalled(client: AgentClient) {
  return snapshot.value.clients.some((item) => item.client === client);
}

function isAgentSettingsLoading(client: AgentClient) {
  return !agentsLoaded.value || agentSettingsLoading.value[client];
}

function copyClientSettings(
  source: ReturnType<typeof createAgentConfiguration>,
  target: ReturnType<typeof createAgentConfiguration>,
  client: AgentClient,
) {
  if (client === "codexCli") {
    Object.assign(target.codexCli, source.codexCli, { features: { ...source.codexCli.features } });
  } else if (client === "chatgpt") {
    Object.assign(target.chatgpt, source.chatgpt, { features: { ...source.chatgpt.features } });
  } else {
    Object.assign(target.claudeCode, source.claudeCode);
  }
}

function codexSettingsPayload(
  codex: typeof agentConfiguration.codexCli | typeof agentConfiguration.chatgpt,
) {
  const { customToken, ...codexSettings } = codex;
  return { ...codexSettings, features: { ...codexSettings.features } };
}

function claudeCodeSettingsPayload(claudeCode = agentConfiguration.claudeCode) {
  return { ...claudeCode };
}

function codexConnection() {
  const codex = activeClient.value === "codexCli" ? settingsDraft.codexCli : settingsDraft.chatgpt;
  const customBaseUrl = codex.customBaseUrl.trim();
  if (isCustomCodexEndpoint.value && customBaseUrl) {
    return {
      kind: "custom",
      baseUrl: customBaseUrl,
      token: codex.customToken,
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
  if (!activeClientDetected.value || !clientSupportsSettings(activeClient.value)) return;
  copyClientSettings(agentConfiguration, settingsDraft, activeClient.value);
  showSettings.value = true;
}

async function selectClient(client: AgentClient) {
  if (showSettings.value && settingsExitRegistration) {
    const canSwitch = await settingsExitRegistration.requestExit();
    if (!canSwitch) return;
  }
  showExtensionCatalog.value = false;
  activeClient.value = client;
  rulesLoaded = clientSupportsSettings(client) && agentSettingsLoaded.value[client];
  if (!agentsLoaded.value) void agentWorkspace.load();
}

function closeSettingsImmediately() {
  showSettings.value = false;
}

function discardSettingsDraft() {
  copyClientSettings(agentConfiguration, settingsDraft, "codexCli");
  copyClientSettings(agentConfiguration, settingsDraft, "chatgpt");
  copyClientSettings(agentConfiguration, settingsDraft, "claudeCode");
  closeSettingsImmediately();
}

function selectExtensionCatalog() {
  showExtensionCatalog.value = true;
  void extensionCatalog.load();
}

function openExtensionDetails(extension: ExtensionPackage) {
  selectedExtension.value = extension;
  showExtensionDetails.value = true;
}

function openExtensionInstall(extension: ExtensionPackage) {
  selectedExtension.value = extension;
  showExtensionInstall.value = true;
}

async function onExtensionInstalled() {
  await agentWorkspace.refresh();
  notifications.success("扩展已安装");
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
  const connection =
    client === "codexCli" || client === "chatgpt"
      ? codexConnection()
      : claudeCodeConnection();
  try {
    await invokeLocalCommand("agent_settings_save", {
      settings:
        client === "codexCli"
          ? { client, settings: codexSettingsPayload(settingsDraft.codexCli) }
          : client === "chatgpt"
            ? { client, settings: codexSettingsPayload(settingsDraft.chatgpt) }
          : {
              client,
              settings: claudeCodeSettingsPayload(settingsDraft.claudeCode),
            },
      connection: connection ? { client, connection } : null,
    });
    copyClientSettings(settingsDraft, agentConfiguration, client);
    showSettings.value = false;
    notifications.success("设置已保存");
    void agentWorkspace.reloadSettings(client);
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

function replaceRulesDraft(client: AgentClient, rules: string) {
  suppressRulesSave = true;
  rulesDraft[client] = rules;
  queueMicrotask(() => {
    suppressRulesSave = false;
  });
}

function discardRulesDraft() {
  if (rulesSaveTimer) clearTimeout(rulesSaveTimer);
  replaceRulesDraft("codexCli", agentConfiguration.codexCli.rules);
  replaceRulesDraft("chatgpt", agentConfiguration.chatgpt.rules);
  replaceRulesDraft("claudeCode", agentConfiguration.claudeCode.rules);
}

async function saveRules(client: AgentClient) {
  rulesSaving.value = true;
  try {
    await invokeLocalCommand("agent_settings_save", {
      settings:
        client === "codexCli"
          ? {
              client,
              settings: codexSettingsPayload({
                ...agentConfiguration.codexCli,
                rules: rulesDraft.codexCli,
              }),
            }
          : client === "chatgpt"
            ? {
                client,
                settings: codexSettingsPayload({
                  ...agentConfiguration.chatgpt,
                  rules: rulesDraft.chatgpt,
                }),
              }
          : {
              client,
              settings: claudeCodeSettingsPayload({
                ...agentConfiguration.claudeCode,
                rules: rulesDraft.claudeCode,
              }),
            },
      connection: null,
    });
    if (client === "codexCli") {
      agentConfiguration.codexCli.rules = rulesDraft.codexCli;
    } else if (client === "chatgpt") {
      agentConfiguration.chatgpt.rules = rulesDraft.chatgpt;
    } else {
      agentConfiguration.claudeCode.rules = rulesDraft.claudeCode;
    }
    notifications.success("规则已保存");
    void agentWorkspace.reloadSettings(client);
  } catch {
    // The local command composable exposes the stable error to this view.
  } finally {
    rulesSaving.value = false;
  }
}

function normalizeBaseUrl(url: string) {
  return url.trim().replace(/\/+$/, "");
}

function managementBaseUrl(relayUrl: string) {
  const normalized = normalizeBaseUrl(relayUrl);
  return normalized.endsWith("/v1") ? normalized : `${normalized}/v1`;
}

async function loadAgentPage(force = false) {
  if (force) {
    rulesLoaded = false;
    void agentWorkspace.refresh();
  } else {
    void agentWorkspace.load(force);
  }
  const endpointsRequest = invokeCommand<RelayEndpoint[]>("endpoints_list")
    .then((value) => {
      endpoints.value = value;
    })
    .catch(() => {
      // The application-level management API status owns endpoint failures.
    });
  const bootstrapRequest = bootstrap.value
    ? Promise.resolve()
    : invokeCommand<BootstrapState>("bootstrap")
        .then(setBootstrap)
        .catch(() => {
          // The application-level management API status owns bootstrap failures.
        });
  await Promise.all([endpointsRequest, bootstrapRequest]);
  if (force && showExtensionCatalog.value) void extensionCatalog.load(true);
}

function hydrateAgentSettings(value: AgentSettings) {
  if (value.client === "codexCli") {
    const { endpointName, baseUrl, customToken, ...codexSettings } =
      value.settings;
    Object.assign(agentConfiguration.codexCli, codexSettings);
    Object.assign(agentConfiguration.codexCli.features, value.settings.features);
    agentConfiguration.codexCli.customBaseUrl = baseUrl ?? "";
    const managementUrl = bootstrap.value?.relay_url
      ? managementBaseUrl(bootstrap.value.relay_url)
      : null;
    const codexEndpoint =
      managementUrl && baseUrl && normalizeBaseUrl(baseUrl) === managementUrl
        ? endpoints.value.find((endpoint) => endpoint.name === endpointName)
        : undefined;
    agentConfiguration.codexCli.endpoint =
      codexEndpoint?.id ?? customEndpointValue;
    agentConfiguration.codexCli.customToken = codexEndpoint
      ? ""
      : (customToken ?? "");
    copyClientSettings(agentConfiguration, settingsDraft, value.client);
    replaceRulesDraft(value.client, agentConfiguration.codexCli.rules);
  } else if (value.client === "chatgpt") {
    const { endpointName, baseUrl, customToken, ...chatgptSettings } =
      value.settings;
    Object.assign(agentConfiguration.chatgpt, chatgptSettings);
    Object.assign(agentConfiguration.chatgpt.features, value.settings.features);
    agentConfiguration.chatgpt.customBaseUrl = baseUrl ?? "";
    const managementUrl = bootstrap.value?.relay_url
      ? managementBaseUrl(bootstrap.value.relay_url)
      : null;
    const chatgptEndpoint =
      managementUrl && baseUrl && normalizeBaseUrl(baseUrl) === managementUrl
        ? endpoints.value.find((endpoint) => endpoint.name === endpointName)
        : undefined;
    agentConfiguration.chatgpt.endpoint =
      chatgptEndpoint?.id ?? customEndpointValue;
    agentConfiguration.chatgpt.customToken = chatgptEndpoint
      ? ""
      : (customToken ?? "");
    copyClientSettings(agentConfiguration, settingsDraft, value.client);
    replaceRulesDraft(value.client, agentConfiguration.chatgpt.rules);
  } else {
    const { baseUrl, endpointToken, ...claudeCodeSettings } = value.settings;
    Object.assign(agentConfiguration.claudeCode, claudeCodeSettings);
    const managementUrl = bootstrap.value?.relay_url
      ? managementBaseUrl(bootstrap.value.relay_url)
      : null;
    const claudeEndpoint =
      managementUrl && baseUrl && normalizeBaseUrl(baseUrl) === managementUrl
        ? endpoints.value.find((endpoint) => endpoint.token === endpointToken)
        : undefined;
    agentConfiguration.claudeCode.endpoint =
      claudeEndpoint?.id ?? customEndpointValue;
    copyClientSettings(agentConfiguration, settingsDraft, "claudeCode");
    replaceRulesDraft("claudeCode", agentConfiguration.claudeCode.rules);
  }

  if (value.client === activeClient.value) {
    void nextTick().then(() => {
      if (value.client === activeClient.value) rulesLoaded = true;
    });
  }
}

async function uninstallAgentItem(item: AgentItem) {
  const kindLabel = { mcp: "MCP", plugin: "插件", skill: "Skill" }[item.kind];
  const confirmed = await confirmAction({
    title: `卸载${kindLabel}`,
    message: `卸载“${item.name}”？`,
    description: "配置及相关本地文件将一并删除，且无法恢复。",
    confirmText: "卸载",
    danger: true,
  });
  if (!confirmed) return;
  try {
    await invokeLocalCommand("agents_remove", {
      client: activeClient.value,
      kind: item.kind,
      name: item.name,
      sourcePath: item.sourcePath,
    });
    await agentWorkspace.refresh();
    notifications.success(`${kindLabel}已卸载`);
  } catch {
    // The local command composable exposes the stable error to this view.
  }
}

onMounted(() => {
  rulesExitRegistration = workspaceExit.register({
    close: discardRulesDraft,
    state: () =>
      rulesSaving.value ? "blocked" : rulesDirty.value ? "discard" : "allow",
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
  () => [cachedAgentSettings.value.codexCli, cachedAgentSettings.value.chatgpt, cachedAgentSettings.value.claudeCode],
  ([codexCli, chatgpt, claudeCode], previous) => {
    if (codexCli && codexCli !== previous?.[0]) hydrateAgentSettings(codexCli);
    if (chatgpt && chatgpt !== previous?.[1]) hydrateAgentSettings(chatgpt);
    if (claudeCode && claudeCode !== previous?.[2]) {
      hydrateAgentSettings(claudeCode);
    }
  },
  { immediate: true },
);

watch(
  snapshot,
  () => {
    if (!activeClientDetected.value) {
  activeClient.value = snapshot.value.clients[0]?.client ?? "codexCli";
    }
  },
  { immediate: true },
);

watch(
  () => rulesDraft.codexCli,
  () => scheduleRulesSave("codexCli"),
);

watch(
  () => rulesDraft.chatgpt,
  () => scheduleRulesSave("chatgpt"),
);

watch(
  () => rulesDraft.claudeCode,
  () => scheduleRulesSave("claudeCode"),
);

onBeforeUnmount(() => {
  if (rulesSaveTimer) clearTimeout(rulesSaveTimer);
  settingsExitRegistration?.unregister();
  rulesExitRegistration?.unregister();
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
          @click="loadAgentPage(true)"
        >
          刷新
        </Button>
      </template>
      <div class="agent-content">
        <List class="agent-client-list" :divided="false">
          <ListItem
            v-for="client in agentClients"
            :key="client.client"
            :active="activeClient === client.client"
            clickable
            @click="selectClient(client.client)"
          >
            <template #prefix>
              <span class="agent-client-icon-frame">
                <img
                  :src="client.icon"
                  :alt="client.label"
                  class="agent-client-icon"
                  :class="{
                    'agent-client-icon--monochrome':
                      client.client === 'codexCli' || client.client === 'chatgpt',
                    'agent-client-icon--uninstalled': !client.installed,
                    'agent-client-icon--loading': isAgentSettingsLoading(
                      client.client,
                    ),
                  }"
                />
                <span
                  v-if="isAgentSettingsLoading(client.client)"
                  class="agent-client-loading"
                >
                  <Icon icon="ph:circle-notch" size="28" />
                </span>
              </span>
            </template>
            <span class="agent-client-identity">
              <span>{{ client.label }}</span>
              <small>{{ client.version }}</small>
            </span>
          </ListItem>
          <ListItem
            class="agent-extension-library"
            :active="showExtensionCatalog"
            clickable
            @click="selectExtensionCatalog"
          >
            <template #prefix>
              <span class="agent-client-icon-frame">
                <Icon icon="ph:storefront" size="24" />
              </span>
            </template>
            <span class="agent-client-identity">
              <span>扩展库</span>
              <small>agents</small>
            </span>
          </ListItem>
        </List>
        <div :key="showExtensionCatalog ? 'extensions' : activeClient" class="agent-main">
          <template v-if="showExtensionCatalog">
            <div class="agent-toolbar">
              <RadioGroup
                v-model="activeExtensionSection"
                :options="extensionSectionOptions"
                variant="button"
              />
            </div>
            <div class="item-results">
              <ExtensionCatalogTable
                :packages="extensionPackages"
                :pending="extensionCatalog.loading.value"
                @detail="openExtensionDetails"
                @install="openExtensionInstall"
              />
            </div>
          </template>
          <template v-else>
            <Loading
              v-if="isAgentSettingsLoading(activeClient)"
              visible
            text="正在读取智能体设置..."
            />
            <section
              v-else-if="!isClientInstalled(activeClient)"
              class="agent-unavailable"
            >
              <Icon icon="ph:download-simple" size="24" />
              <p>未检测到本机安装</p>
            </section>
            <template v-else>
              <div class="agent-toolbar">
                <RadioGroup
                  v-model="activeSection"
                  :options="sectionOptions"
                  variant="button"
                />
                <div class="agent-toolbar__actions">
                  <Button
                    icon="ph:sliders-horizontal"
                    aria-label="配置"
                    title="配置"
                  @click="openSettings"
                  >
                  配置
                </Button>
                </div>
              </div>
              <template v-if="activeSection === 'rules'">
                <AgentRulesEditor
                  v-if="activeClient === 'codexCli'"
                  v-model="rulesDraft.codexCli"
                />
                <AgentRulesEditor
                  v-else-if="activeClient === 'chatgpt'"
                  v-model="rulesDraft.chatgpt"
                />
                <AgentRulesEditor v-else v-model="rulesDraft.claudeCode" />
              </template>
              <div v-else class="item-results">
                <AgentItemList
                  :items="sectionItems"
                  :pending="pending"
                  @uninstall="uninstallAgentItem"
                />
              </div>
            </template>
          </template>
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
    <form
      id="agent-settings-form"
      class="agent-settings-form"
      @submit.prevent="saveSettings"
    >
      <CodexSettingsForm
        v-if="activeClient === 'codexCli'"
        v-model="settingsDraft.codexCli"
        :visible="showSettings"
        :endpoint-options="endpointOptions"
        :model-options="modelOptions"
        :custom-endpoint-value="customEndpointValue"
      />
      <ChatGptSettingsForm
        v-else-if="activeClient === 'chatgpt'"
        v-model="settingsDraft.chatgpt"
        :visible="showSettings"
        :endpoint-options="endpointOptions"
        :model-options="modelOptions"
        :custom-endpoint-value="customEndpointValue"
      />
      <ClaudeCodeSettingsForm
        v-else
        v-model="settingsDraft.claudeCode"
        :endpoint-options="endpointOptions"
        :model-options="modelOptions"
      />
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
  <ExtensionDetailDrawer
    v-model:visible="showExtensionDetails"
    :extension="selectedExtension"
  />
  <ExtensionInstallModal
    v-model:visible="showExtensionInstall"
    :extension="selectedExtension"
    :detected-clients="snapshot.clients.map((client) => client.client)"
    @installed="onExtensionInstalled"
  />
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

.agent-client-list :deep(.st-list-item > div:first-child) {
  display: flex;
  width: 40px;
  height: 40px;
  align-items: center;
  justify-content: center;
}

.agent-extension-library {
  margin-top: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--st-border-divider);
}

.agent-client-icon-frame {
  display: grid;
  position: relative;
  width: 40px;
  height: 40px;
  place-items: center;
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-md);
  background: var(--st-bg-elevated);
}

.agent-client-icon {
  display: block;
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.agent-client-icon--monochrome {
  filter: var(--pr-monochrome-icon-filter);
}

.agent-client-icon--uninstalled,
.agent-client-icon--loading {
  filter: grayscale(1);
  opacity: 0.45;
}

.agent-client-loading {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  pointer-events: none;
  color: var(--st-text-primary);
  animation: agent-loading-spin 800ms linear infinite;
}

.agent-client-identity {
  display: grid;
  gap: 2px;
}

.agent-client-identity small {
  color: var(--st-text-secondary);
  font-size: 12px;
}

.agent-main {
  display: flex;
  position: relative;
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

.agent-unavailable {
  display: grid;
  flex: 1;
  place-content: center;
  justify-items: center;
  gap: var(--spacing-sm);
  color: var(--st-text-secondary);
}

@keyframes agent-loading-spin {
  to {
    transform: rotate(360deg);
  }
}

.agent-unavailable p {
  margin: 0;
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
</style>
