<script setup lang="ts">
import { Avatar as DiceBearAvatar, Style } from "@dicebear/core";
import cutouts from "@dicebear/styles/cutouts.json";
import { Avatar, Sidebar, SidebarItem } from "@stellar/ui";
import DashboardStatusbar from "~/components/dashboard/DashboardStatusbar.vue";
import { type BootstrapState, useRelayStore } from "~/stores/relay";

defineProps<{
  relayUrl: string | null;
}>();

const route = useRoute();
const { invokeCommand } = useRelayCommand();
const { bootstrap, setBootstrap } = useRelayStore();
const navigation = [
  { label: "仪表盘", path: "/", icon: "ph:squares-four" },
  { label: "供应商", path: "/providers", icon: "ph:plugs-connected" },
  { label: "接入点", path: "/endpoints", icon: "ph:key" },
  { label: "智能体", path: "/agents", icon: "ph:robot" },
  { label: "活动", path: "/stats", icon: "ph:chart-line-up" },
];
const displayName = computed(() => bootstrap.value?.display_name ?? "当前用户");
const cutoutsStyle = new Style(cutouts);
const avatarSrc = computed(() =>
  new DiceBearAvatar(cutoutsStyle, {
    seed: bootstrap.value?.avatar_seed ?? "current-user",
  }).toDataUri(),
);

onMounted(async () => {
  if (bootstrap.value) return;
  try {
    setBootstrap(await invokeCommand<BootstrapState>("bootstrap"));
  } catch {
    // The command composable exposes the management API error.
  }
});

</script>

<template>
  <div class="dashboard-shell">
    <div class="dashboard-body">
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
          <div class="dashboard-user" :title="displayName">
            <Avatar
              :src="avatarSrc"
              :alt="displayName"
              size="large"
              shape="circle"
            />
            <span>{{ displayName }}</span>
          </div>
        </template>
      </Sidebar>

      <main class="dashboard-main">
        <slot />
      </main>
    </div>
    <DashboardStatusbar :relay-url="relayUrl" />
  </div>
</template>

<style scoped>
.dashboard-shell {
  display: grid;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  grid-template-rows: minmax(0, 1fr) var(--pr-statusbar-height);
  background: var(--st-bg-panel);
  color: var(--st-text-primary);
}

.dashboard-body {
  display: grid;
  min-width: 0;
  min-height: 0;
  grid-template-columns: auto minmax(0, 1fr);
}

.dashboard-main {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  background: var(--st-bg-base);
}

.dashboard-user {
  display: grid;
  width: 100%;
  justify-items: center;
  gap: 4px;
  padding: var(--spacing-sm) 2px var(--spacing-md);
  color: var(--st-text-secondary);
  font-size: 11px;
}

.dashboard-user span {
  width: 100%;
  overflow: hidden;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
