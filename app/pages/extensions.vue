<script setup lang="ts">
defineOptions({ name: "ExtensionsPage" });

import { Button, RadioGroup } from "@stellar/ui";
import type { AgentClient, ExtensionCatalogKind } from "~/types/relay";
import ExtensionCatalogTable from "~/components/extensions/CatalogTable.vue";
import ExtensionDetailDrawer from "~/components/extensions/DetailDrawer.vue";
import ExtensionInstallDrawer from "~/components/extensions/install/Drawer.vue";
import PanelSection from "~/components/shell/layout/PanelSection.vue";
import { useRelayStore } from "~/stores/relay";
import { sortAgentClients } from "~/utils/agent/client";

const { bootstrap } = useRelayStore();
const agentWorkspace = useAgentWorkspace();
const extensionLibrary = useExtensionLibrary({
  relayUrl: computed(() => bootstrap.value?.relay_url ?? null),
});
const {
  activeSection: extensionSection,
  loading: extensionLoading,
  openDetails: openExtensionDetails,
  openInstall: openExtensionInstall,
  packages: extensionPackages,
  refresh: refreshExtensions,
  selected: selectedExtension,
  showDetails: showExtensionDetails,
  showInstall: showExtensionInstall,
  updates: extensionUpdates,
} = extensionLibrary;
const {
  activeJobs,
  batchUpdating,
  install: installExtension,
  updateAll: updateExtensions,
} = useExtensionInstallJobs();

const sectionOptions: Array<{
  value: ExtensionCatalogKind;
  label: string;
  icon: string;
}> = [
  { value: "rule", label: "规则", icon: "ph:notebook" },
  { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
];
const detectedClients = computed<AgentClient[]>(() =>
  sortAgentClients(
    agentWorkspace.clientStatuses.value.map(({ client, installed }) => ({
      client,
      installed,
    })),
  )
    .filter(({ installed }) => installed)
    .map(({ client }) => client),
);

function updateSection(value: string | number | boolean | null) {
  if (typeof value === "string") {
    extensionSection.value = value as ExtensionCatalogKind;
  }
}

async function refreshExtensionsPage() {
  try {
    await Promise.all([
      refreshExtensions("rule"),
      refreshExtensions("plugin"),
      agentWorkspace.refreshClientStatuses(),
    ]);
  } catch {
    // 目录读取失败由列表空态和管理接口状态呈现。
  }
}

async function updateAllExtensions() {
  await updateExtensions(extensionPackages.value);
}

onMounted(() => {
  // 安装目标来自本机检测结果，进入扩展页面时重新确认已安装客户端。
  void agentWorkspace.refreshClientStatuses();
});
</script>

<template>
  <main class="page-dashboard">
    <PanelSection title="扩展">
      <template #header-actions>
        <Button
          semantic="primary"
          variant="solid"
          icon="ph:arrows-clockwise"
          :disabled="extensionLoading"
          aria-label="刷新"
          title="刷新"
          @click="refreshExtensionsPage"
        >
          {{ extensionLoading ? "刷新中..." : "刷新" }}
        </Button>
      </template>
      <div class="extension-library">
        <div class="extension-library__toolbar">
          <RadioGroup
            :model-value="extensionSection"
            :options="sectionOptions"
            variant="button"
            @update:model-value="updateSection"
          />
          <div
            v-if="extensionUpdates > 0 || batchUpdating"
            class="extension-library__update"
          >
            <Button
              icon="ph:arrow-circle-down"
              :disabled="batchUpdating"
              @click="updateAllExtensions"
            >
              {{ batchUpdating ? "更新中..." : "全部更新" }}
            </Button>
          </div>
        </div>
        <ExtensionCatalogTable
          :packages="extensionPackages"
          :pending="extensionLoading"
          :installing="activeJobs"
          @detail="openExtensionDetails"
          @install="openExtensionInstall"
        />
      </div>
    </PanelSection>
    <ExtensionDetailDrawer
      v-model:visible="showExtensionDetails"
      :extension="selectedExtension"
    />
    <ExtensionInstallDrawer
      v-model:visible="showExtensionInstall"
      :extension="selectedExtension"
      :detected-clients="detectedClients"
      @install="installExtension"
    />
  </main>
</template>

<style scoped>
.page-dashboard {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--pr-dashboard-padding);
}

.extension-library {
  display: flex;
  flex: 1;
  min-height: 0;
  flex-direction: column;
  gap: var(--spacing-md);
}

.extension-library__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.extension-library__update {
  display: flex;
  flex: 0 0 auto;
}
</style>
