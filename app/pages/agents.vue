<script setup lang="ts">
import { Button } from "@stellar/ui";
import type { AgentClient } from "~/types/relay";
import AgentClientContent from "~/components/agents/workspace/Content.vue";
import AgentSettingsDrawer from "~/components/agents/settings/Drawer.vue";
import AgentSidebar from "~/components/agents/workspace/Sidebar.vue";
import ExtensionDetailDrawer from "~/components/extensions/DetailDrawer.vue";
import ExtensionInstallDrawer from "~/components/extensions/install/Drawer.vue";
import ExtensionLibraryContent from "~/components/extensions/library/Content.vue";
import PanelSection from "~/components/shell/layout/PanelSection.vue";

const workspace = useAgentWorkspace();
const {
  clientStatusesLoaded,
  clientStatusesLoading,
  settings: cachedAgentSettings,
} = workspace;
const selection = useAgentPageSelection(workspace);
const {
  activateClient,
  activateExtensions,
  activeClient,
  activeClientDetected,
  activeSection,
  activeWorkspace,
  agentClients,
  availableSectionOptions,
  isAgentContentLoading,
  sectionItems,
} = selection;
const pageData = useAgentPageData();
const {
  bootstrap,
  endpoints,
  load: loadAgentPage,
  pending: pagePending,
} = pageData;
const { pending: agentsPending, invokeLocalCommand } = useLocalCommand();
const pending = computed(() => agentsPending.value || pagePending.value);

const agentSettings = useAgentSettings({
  activeClient,
  bootstrap,
  endpoints,
  reloadSettings: workspace.reloadSettings,
  save: (request) => invokeLocalCommand("agent_settings_save", request),
  settings: cachedAgentSettings,
});
const {
  close: closeSettings,
  configuration: agentConfiguration,
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
  reloadSettings: workspace.reloadSettings,
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
const extensionLibrary = useExtensionLibrary({
  refreshClient: () => workspace.refreshClient(activeClient.value),
  relayUrl: computed(() => bootstrap.value?.relay_url ?? null),
  visible: computed(() => activeWorkspace.value === "extensions"),
});
const {
  activeSection: extensionSection,
  installed: extensionInstalled,
  loading: extensionLoading,
  openDetails: openExtensionDetails,
  openInstall: openExtensionInstall,
  packages: extensionPackages,
  refresh: refreshExtensions,
  selected: selectedExtension,
  showDetails: showExtensionDetails,
  showInstall: showExtensionInstall,
  totalUpdates: totalExtensionUpdates,
  updateAll: updateAllExtensions,
  updates: extensionUpdates,
  updating: extensionUpdating,
} = extensionLibrary;
const itemActions = useAgentItemActions({
  client: activeClient,
  refreshClient: workspace.refreshClient,
  remove: (request) => invokeLocalCommand("agents_remove", request),
});

const workspaceExit = useWorkspaceExitGuard();
type ExitRegistration = ReturnType<typeof workspaceExit.register>;
let settingsExitRegistration: ExitRegistration | undefined;
let rulesExitRegistration: ExitRegistration | undefined;

async function selectClient(client: AgentClient) {
  if (showSettings.value && settingsExitRegistration) {
    const canSwitch = await settingsExitRegistration.requestExit();
    if (!canSwitch) return;
  }
  activateClient(client);
  agentRules.pause();
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

function refreshAgentClients() {
  void workspace.refreshClientStatuses();
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
          semantic="primary"
          variant="solid"
          icon="ph:arrows-clockwise"
          :disabled="clientStatusesLoading"
          aria-label="刷新"
          title="刷新"
          @click="refreshAgentClients"
        >
          {{ clientStatusesLoading ? "刷新中..." : "刷新" }}
        </Button>
      </template>
      <div class="agent-content">
        <AgentSidebar
          :active-workspace="activeWorkspace"
          :clients="agentClients"
          :status-loading="clientStatusesLoading"
          :total-extension-updates="totalExtensionUpdates"
          @select-client="selectClient"
          @select-extensions="activateExtensions"
        />
        <AgentClientContent
          v-if="activeWorkspace !== 'extensions'"
          v-model:active-section="activeSection"
          v-model:rules="activeRules"
          :client-content-loading="isAgentContentLoading(activeClient)"
          :client-installed="activeClientDetected"
          :client-statuses-loaded="clientStatusesLoaded"
          :item-pending="pending"
          :section-items="sectionItems"
          :section-options="availableSectionOptions"
          @open-settings="openSettings(activeClientDetected)"
          @uninstall="itemActions.uninstall"
        />
        <ExtensionLibraryContent
          v-else
          v-model:active-section="extensionSection"
          :loading="extensionLoading"
          :packages="extensionPackages"
          :updates="extensionUpdates"
          :updating="extensionUpdating"
          @detail="openExtensionDetails"
          @install="openExtensionInstall"
          @update-all="updateAllExtensions"
        />
      </div>
    </PanelSection>
  </main>
  <AgentSettingsDrawer
    :visible="showSettings"
    v-model:draft="settingsDraft"
    :active-client="activeClient"
    :blocked="pending || settingsDirty"
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
  <ExtensionInstallDrawer
    v-model:visible="showExtensionInstall"
    :extension="selectedExtension"
    :detected-clients="
      agentClients
        .filter((client) => client.installed)
        .map((client) => client.client)
    "
    @installed="extensionInstalled"
    @refreshed="refreshExtensions"
  />
</template>

<style scoped src="./agents.css"></style>
