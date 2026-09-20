<script setup lang="ts">
import { Avatar, Badge, Sidebar, SidebarItem } from "@stellar/ui";
import DashboardStatusbar from "~/components/dashboard/DashboardStatusbar.vue";
import { identityAvatarSrc } from "~/utils/identityAvatar";
import { type BootstrapState, useRelayStore } from "~/stores/relay";

defineProps<{
  relayUrl: string | null;
}>();

const route = useRoute();
const { invokeCommand } = useRelayCommand();
const { bootstrap, setBootstrap } = useRelayStore();
const totalExtensionUpdates = useExtensionCatalog().totalUpdateCount;
const navigation = [
  { label: "仪表盘", path: "/", icon: "ph:squares-four" },
  { label: "供应商", path: "/providers", icon: "ph:plugs-connected" },
  { label: "接入点", path: "/endpoints", icon: "ph:key" },
  { label: "智能体", path: "/agents", icon: "ph:robot" },
  { label: "活动", path: "/stats", icon: "ph:chart-line-up" },
];
const displayName = computed(() => bootstrap.value?.display_name ?? "当前用户");
const avatarSrc = computed(() =>
  identityAvatarSrc(bootstrap.value?.identity_id ?? "current-user"),
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
        >
          <template
            v-if="item.path === '/agents' && totalExtensionUpdates > 0"
            #badge
          >
            <Badge
              class="dashboard-nav-notice"
              dot
              semantic="error"
              size="small"
              aria-label="有可更新的扩展"
            />
          </template>
        </SidebarItem>
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

/* 导航项根节点是 relative，红点贴在选中区域右上角靠里一点，不依赖图标尺寸与排版。 */
.dashboard-nav-notice {
  position: absolute;
  top: 6px;
  right: 6px;
}
</style>
