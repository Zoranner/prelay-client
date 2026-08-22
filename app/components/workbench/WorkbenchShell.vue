<script setup lang="ts">
import { Sidebar, SidebarItem } from "stellar-ui";
import WorkbenchStatusbar from "~/components/workbench/WorkbenchStatusbar.vue";

defineProps<{
  relayUrl: string | null;
}>();

const route = useRoute();
const desktopPreferencesDialog = useDesktopPreferencesDialog();
const navigation = [
  { label: "工作台", path: "/", icon: "ph:squares-four" },
  { label: "供应商", path: "/providers", icon: "ph:plugs-connected" },
  { label: "接入点", path: "/endpoints", icon: "ph:key" },
  { label: "活动", path: "/stats", icon: "ph:chart-line-up" },
];

function openDesktopPreferences() {
  desktopPreferencesDialog.open();
}
</script>

<template>
  <div class="workbench-shell">
    <div class="workbench-body">
      <Sidebar variant="rail" :show-header="false">
        <SidebarItem
          v-for="item in navigation"
          :key="item.path"
          :active="route.path === item.path"
          :icon="item.icon"
          :label="item.label"
          :to="item.path"
        />
        <template #footer>
          <SidebarItem
            icon="ph:gear-six"
            label="设置"
            @click="openDesktopPreferences"
          />
        </template>
      </Sidebar>

      <main class="workbench-main">
        <slot />
      </main>
    </div>
    <WorkbenchStatusbar :relay-url="relayUrl" />
  </div>
</template>

<style scoped>
.workbench-shell {
  display: grid;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  grid-template-rows: minmax(0, 1fr) var(--pr-statusbar-height);
  background: var(--st-bg-panel);
  color: var(--st-text-primary);
}

.workbench-body {
  display: grid;
  min-width: 0;
  min-height: 0;
  grid-template-columns: auto minmax(0, 1fr);
}

.workbench-main {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  background: var(--st-bg-base);
}
</style>
