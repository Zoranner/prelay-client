<script setup lang="ts">
import { Button, Icon, useConfirm, useNotification } from "@stellar/ui";
import type {
  AgentClient,
  AgentItem,
  AgentItemKind,
  BootstrapState,
  ExtensionKind,
  ExtensionPackage,
  RelayEndpoint,
} from "~/stores/relay";
import { useRelayStore } from "~/stores/relay";
import { agentClientDefinitions } from "~/utils/agentClient";
import AgentSidebar from "~/components/agents/AgentSidebar.vue";
import AgentWorkspaceContent from "~/components/agents/AgentWorkspaceContent.vue";
import AgentSettingsDrawer from "~/components/agents/AgentSettingsDrawer.vue";
import ExtensionDetailDrawer from "~/components/extensions/ExtensionDetailDrawer.vue";
import ExtensionInstallModal from "~/components/extensions/ExtensionInstallModal.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

type AgentSection = "rules" | AgentItemKind;
type AgentWorkspace = AgentClient | "extensions";

const { pending: agentsPending, invokeLocalCommand } = useLocalCommand();
const { pending: endpointsPending, invokeCommand } = useRelayCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
const workspaceExit = useWorkspaceExitGuard();
const { bootstrap, setBootstrap } = useRelayStore();
const agentWorkspace = useAgentWorkspace();
const extensionCatalog = useExtensionCatalog();
const {
  clientStatuses,
  clientStatusesLoaded,
  clientStatusesLoading,
  clientItems,
  itemsLoading,
  settings: cachedAgentSettings,
  settingsLoading: agentSettingsLoading,
} = agentWorkspace;
const endpoints = ref<RelayEndpoint[]>([]);
const activeWorkspace = ref<AgentWorkspace>("codexCli");
const lastActiveClient = ref<AgentClient>("codexCli");
const activeClient = computed<AgentClient>(() =>
  activeWorkspace.value === "extensions"
    ? lastActiveClient.value
    : activeWorkspace.value,
);
const activeSection = ref<AgentSection>("rules");
const activeExtensionSection = ref<ExtensionKind>("rule");
const selectedExtension = ref<ExtensionPackage | null>(null);
const showExtensionDetails = ref(false);
const showExtensionInstall = ref(false);
let settingsExitRegistration:
  ReturnType<typeof workspaceExit.register> | undefined;
let rulesExitRegistration:
  ReturnType<typeof workspaceExit.register> | undefined;

const agentSettings = useAgentSettings({
  activeClient,
  bootstrap,
  endpoints,
  reloadSettings: agentWorkspace.reloadSettings,
  save: (request) => invokeLocalCommand("agent_settings_save", request),
  settings: cachedAgentSettings,
});
const {
  close: closeSettings,
  configuration: agentConfiguration,
  customEndpointValue,
  dirty: settingsDirty,
  discard: discardSettingsDraft,
  draft: settingsDraft,
  endpointOptions,
  modelOptions,
  open: openSettings,
  save: saveAgentSettings,
  showSettings,
} = agentSettings;
const agentRules = useAgentRules({
  configuration: agentConfiguration,
  reloadSettings: agentWorkspace.reloadSettings,
  save: (request) => invokeLocalCommand("agent_settings_save", request),
});
useAgentRulesHydration({
  activeClient,
  settings: cachedAgentSettings,
  hydrate: agentRules.hydrate,
});
const activeRules = computed({
  get: () => agentRules.draft[activeClient.value],
  set: (rules: string) => {
    agentRules.draft[activeClient.value] = rules;
  },
});

const agentClients = computed(() =>
  agentClientDefinitions.map((definition) => {
    const status = clientStatuses.value.find(
      ({ client }) => client === definition.client,
    );
    return {
      ...definition,
      installed: clientStatusesLoaded.value && Boolean(status?.installed),
      version: clientStatusesLoaded.value ? (status?.version ?? "-") : "-",
    };
  }),
);
const sectionOptions: Array<{
  value: AgentSection;
  label: string;
  icon: string;
}> = [
  { value: "rules", label: "规则", icon: "ph:notebook" },
  { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
  { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
  { value: "skill", label: "Skill", icon: "ph:book-open-text" },
];
const availableSectionOptions = computed(() => {
  const definition = agentClientDefinitions.find(
    (candidate) => candidate.client === activeClient.value,
  );
  return sectionOptions.filter(({ value }) =>
    definition?.sections.includes(value),
  );
});
const extensionSectionOptions: Array<{
  value: ExtensionKind;
  label: string;
  icon: string;
}> = [
  { value: "rule", label: "规则", icon: "ph:notebook" },
  { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
  { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
  { value: "skill", label: "Skill", icon: "ph:book-open-text" },
];
const activeItems = computed(
  () => clientItems.value[activeClient.value]?.items ?? [],
);
const activeKind = computed<AgentItemKind | null>(() =>
  activeSection.value === "rules" ? null : activeSection.value,
);
const sectionItems = computed(() =>
  activeKind.value
    ? activeItems.value.filter((item) => item.kind === activeKind.value)
    : [],
);
const extensionPackages = computed(
  () => extensionCatalog.catalogs.value[activeExtensionSection.value].packages,
);
const pending = computed(() => agentsPending.value || endpointsPending.value);
const activeClientDetected = computed(() =>
  isClientInstalled(activeClient.value),
);

function isClientInstalled(client: AgentClient) {
  return clientStatuses.value.some(
    (status) => status.client === client && status.installed,
  );
}

function isAgentContentLoading(client: AgentClient) {
  return itemsLoading.value[client] || agentSettingsLoading.value[client];
}

async function selectClient(client: AgentClient) {
  if (showSettings.value && settingsExitRegistration) {
    const canSwitch = await settingsExitRegistration.requestExit();
    if (!canSwitch) return;
  }
  lastActiveClient.value = client;
  activeWorkspace.value = client;
  agentRules.pause();
  if (
    !availableSectionOptions.value.some(
      ({ value }) => value === activeSection.value,
    )
  ) {
    activeSection.value = availableSectionOptions.value[0]?.value ?? "mcp";
  }
  if (clientStatusesLoaded.value && isClientInstalled(client)) {
    void agentWorkspace.refreshClient(client);
  }
}

function selectExtensionCatalog() {
  activeWorkspace.value = "extensions";
  void extensionCatalog.load(activeExtensionSection.value);
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
  await agentWorkspace.refreshClient(lastActiveClient.value);
  notifications.success("扩展已安装");
}

function requestCloseSettings() {
  if (settingsExitRegistration) {
    void settingsExitRegistration.requestExit();
    return;
  }
  closeSettings();
}

function updateSettingsVisibility(visible: boolean) {
  if (visible) showSettings.value = true;
  else requestCloseSettings();
}

async function saveSettings() {
  try {
    await saveAgentSettings();
  } catch {
    // The local command composable exposes the stable error to this view.
  }
}

async function loadAgentPage() {
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
}

function refreshAgentClients() {
  void agentWorkspace.refreshClientStatuses();
}

function refreshActiveClient() {
  if (
    activeWorkspace.value !== "extensions" &&
    clientStatusesLoaded.value &&
    activeClientDetected.value
  ) {
    void agentWorkspace.refreshClient(activeClient.value);
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
    await agentWorkspace.refreshClient(activeClient.value);
    notifications.success(`${kindLabel}已卸载`);
  } catch {
    // The local command composable exposes the stable error to this view.
  }
}

onMounted(() => {
  rulesExitRegistration = workspaceExit.register({
    close: agentRules.discard,
    state: () =>
      agentRules.saving.value
        ? "blocked"
        : agentRules.dirty.value
          ? "discard"
          : "allow",
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
  clientStatuses,
  () => {
    if (activeWorkspace.value !== "extensions" && !activeClientDetected.value) {
      const fallback =
        clientStatuses.value.find((status) => status.installed)?.client ??
        "codexCli";
      lastActiveClient.value = fallback;
      activeWorkspace.value = fallback;
    }
  },
  { immediate: true },
);

watch([clientStatusesLoaded, activeWorkspace], refreshActiveClient, {
  immediate: true,
});

watch(activeExtensionSection, (kind) => {
  if (activeWorkspace.value === "extensions") void extensionCatalog.load(kind);
});

watch(
  () => bootstrap.value?.relay_url,
  () => {
    extensionCatalog.invalidate();
    if (activeWorkspace.value === "extensions") {
      void extensionCatalog.load(activeExtensionSection.value);
    }
  },
);

onBeforeUnmount(() => {
  agentRules.stop();
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
          :disabled="clientStatusesLoading"
          aria-label="刷新"
          title="刷新"
          @click="refreshAgentClients"
        >
          <template #icon>
            <Icon
              :icon="
                clientStatusesLoading
                  ? 'ph:circle-notch'
                  : 'ph:arrows-clockwise'
              "
              :class="{ 'agent-refresh-icon--loading': clientStatusesLoading }"
            />
          </template>
          刷新
        </Button>
      </template>
      <div class="agent-content">
        <AgentSidebar
          :active-workspace="activeWorkspace"
          :clients="agentClients"
          :status-loading="clientStatusesLoading"
          @select-client="selectClient"
          @select-extensions="selectExtensionCatalog"
        />
        <AgentWorkspaceContent
          v-model:active-extension-section="activeExtensionSection"
          v-model:active-section="activeSection"
          v-model:rules="activeRules"
          :active-client="activeClient"
          :client-content-loading="isAgentContentLoading(activeClient)"
          :client-installed="activeClientDetected"
          :client-statuses-loaded="clientStatusesLoaded"
          :extension-loading="
            extensionCatalog.loading.value[activeExtensionSection]
          "
          :extension-packages="extensionPackages"
          :extension-section-options="extensionSectionOptions"
          :item-pending="pending"
          :section-items="sectionItems"
          :section-options="availableSectionOptions"
          :workspace="activeWorkspace"
          @detail="openExtensionDetails"
          @install="openExtensionInstall"
          @open-settings="openSettings(activeClientDetected)"
          @uninstall="uninstallAgentItem"
        />
      </div>
    </PanelSection>
  </main>
  <AgentSettingsDrawer
    :visible="showSettings"
    v-model:draft="settingsDraft"
    :active-client="activeClient"
    :blocked="pending || settingsDirty"
    :custom-endpoint-value="customEndpointValue"
    :endpoint-options="endpointOptions"
    :model-options="modelOptions"
    :pending="pending"
    @close="requestCloseSettings"
    @save="saveSettings"
    @update:visible="updateSettingsVisibility"
  />
  <ExtensionDetailDrawer
    v-model:visible="showExtensionDetails"
    :extension="selectedExtension"
  />
  <ExtensionInstallModal
    v-model:visible="showExtensionInstall"
    :extension="selectedExtension"
    :detected-clients="
      agentClients
        .filter((client) => client.installed)
        .map((client) => client.client)
    "
    @installed="onExtensionInstalled"
  />
</template>

<style scoped src="./agents.css"></style>
