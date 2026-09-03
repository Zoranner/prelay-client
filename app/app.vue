<script setup lang="ts">
import { isTauri } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from "@tauri-apps/api/event";
import { Button, NotificationContainer, Result } from "@stellar/ui";
import type { ProviderCatalogResponse } from "~/stores/relay";
import AppTitlebar from "~/components/shell/AppTitlebar.vue";
import ClientUpdateDialog from "~/components/settings/ClientUpdateDialog.vue";
import DesktopPreferencesDialog from "~/components/settings/DesktopPreferencesDialog.vue";
import DashboardShell from "~/components/dashboard/DashboardShell.vue";
import { setModelCatalog } from "~/utils/modelCatalog";

const managementApi = useRelayManagementApiStatus();
const { invokeCommand } = useRelayCommand();
const managementApiError = computed(() => managementApi.error.value);
const relaySettings = useRelaySettings();
const desktopPreferences = useDesktopPreferences();
const desktopPreferencesDialog = useDesktopPreferencesDialog();
const clientUpdate = useClientUpdate();
const agentWorkspace = useAgentWorkspace();
const { visible: desktopPreferencesVisible } = desktopPreferencesDialog;
const relayUrl = computed(() => relaySettings.relayUrl.value);
const route = useRoute();
const isSetupRoute = computed(() => route.path === "/setup");
const canShowManagementError = computed(() => route.path !== "/setup");
const workspacePageKey = ref(0);
let modelCatalogUrl: string | null = null;
let modelCatalogRequestId = 0;
let unlistenTraySettings: UnlistenFn | undefined;

onMounted(async () => {
  managementApi.clear();
  await desktopPreferences.load();
  if (!isDesktopRuntime()) return;

  void agentWorkspace.refreshClientStatuses();
  void tryLoadModelCatalog();
  if (relayUrl.value) void clientUpdate.checkAndDownload();

  unlistenTraySettings = await listen("tray:open-settings", () => {
    desktopPreferencesDialog.open();
  });
});

watch(relayUrl, (url) => {
  if (!url) {
    modelCatalogUrl = null;
    modelCatalogRequestId += 1;
    setModelCatalog();
    return;
  }

  void tryLoadModelCatalog(url);
});

onUnmounted(() => {
  unlistenTraySettings?.();
});

function reloadApplication() {
  managementApi.clear();
  workspacePageKey.value += 1;
}

function switchRelayAddress() {
  managementApi.clear();
  void navigateTo("/setup?change=1");
}

async function tryLoadModelCatalog(url = relayUrl.value) {
  if (!isDesktopRuntime() || !url || modelCatalogUrl === url) return;

  modelCatalogUrl = url;
  const requestId = ++modelCatalogRequestId;
  setModelCatalog();

  try {
    const catalog =
      await invokeCommand<ProviderCatalogResponse>("catalog_models_get");
    if (modelCatalogUrl === url && requestId === modelCatalogRequestId) {
      setModelCatalog(catalog);
    }
  } catch {
    if (modelCatalogUrl === url && requestId === modelCatalogRequestId) {
      setModelCatalog();
    }
  }
}

function isDesktopRuntime() {
  return isTauri() || "__TAURI_INTERNALS__" in globalThis;
}
</script>

<template>
  <div class="app-root">
    <AppTitlebar />
    <DashboardShell v-if="!isSetupRoute" :relay-url="relayUrl">
      <NuxtPage :key="workspacePageKey" />
    </DashboardShell>
    <NuxtPage v-else />
    <div
      v-if="managementApiError && canShowManagementError"
      class="app-error-state"
    >
      <Result
        status="error"
        title="无法连接服务地址"
        description="当前无法访问配置的服务地址。"
      >
        <p class="app-error-detail">{{ managementApiError.message }}</p>
        <Button @click="switchRelayAddress">切换服务地址</Button>
        <Button variant="primary" @click="reloadApplication"> 重新加载 </Button>
      </Result>
    </div>
    <DesktopPreferencesDialog v-model:visible="desktopPreferencesVisible" />
    <ClientUpdateDialog
      v-model:visible="clientUpdate.visible.value"
      v-model:version="clientUpdate.version.value"
    />
    <NotificationContainer position="top-right" :max="5" />
  </div>
</template>

<style scoped>
.app-root {
  display: grid;
  height: 100dvh;
  min-height: 0;
  grid-template-rows: var(--pr-titlebar-height) minmax(0, 1fr);
  overflow: hidden;
  border: 1px solid var(--st-border);
  background: var(--st-bg-panel);
  color: var(--st-text-primary);
}

.app-error-state {
  position: fixed;
  z-index: 100;
  inset: var(--pr-titlebar-height) 0 0;
  display: grid;
  place-items: center;
  padding: var(--spacing-xl);
  background: color-mix(in srgb, var(--st-bg-base) 88%, transparent);
}

.app-error-detail {
  margin: 0 0 var(--spacing-md);
  color: var(--st-text-muted);
  font-family: var(--font-family-mono);
  font-size: 12px;
  overflow-wrap: anywhere;
}
</style>
