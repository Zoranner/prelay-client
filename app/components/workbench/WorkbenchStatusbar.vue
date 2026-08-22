<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { Icon } from "stellar-ui";

const props = defineProps<{
  relayUrl: string | null;
}>();

const managementApi = useRelayManagementApiStatus();
const clientVersion = ref<string | null>(null);
const isConnected = computed(
  () => Boolean(props.relayUrl) && !managementApi.error.value,
);
const statusTitle = computed(() =>
  isConnected.value ? "已连接管理服务" : "管理服务不可用",
);

function switchRelayAddress() {
  managementApi.clear();
  void navigateTo("/setup?change=1");
}

onMounted(async () => {
  try {
    clientVersion.value = await getVersion();
  } catch {
    // The browser dev server does not expose the Tauri app API.
  }
});
</script>

<template>
  <footer class="workbench-statusbar">
    <div class="workbench-statusbar__connection">
      <span
        class="workbench-statusbar__dot"
        :class="{ 'workbench-statusbar__dot--connected': isConnected }"
        :title="statusTitle"
      />
      <span
        class="workbench-statusbar__address"
        :title="relayUrl ?? '未配置接入点地址'"
      >
        {{ relayUrl ?? "未配置接入点地址" }}
      </span>
      <button
        class="workbench-statusbar__switch"
        type="button"
        aria-label="切换接入点地址"
        title="切换接入点地址"
        @click="switchRelayAddress"
      >
        <Icon icon="ph:arrows-left-right" />
      </button>
    </div>
    <span class="workbench-statusbar__version"
      >v{{ clientVersion ?? "-" }}</span
    >
  </footer>
</template>

<style scoped>
.workbench-statusbar {
  display: flex;
  min-width: 0;
  height: var(--pr-statusbar-height);
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 0 var(--spacing-md);
  border-top: 1px solid var(--st-border-divider);
  background: var(--st-bg-header);
  color: var(--st-text-muted);
  font-family: var(--font-family-mono);
  font-size: 12px;
}

.workbench-statusbar__connection {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-sm);
}

.workbench-statusbar__dot {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  border-radius: 999px;
  background: var(--st-danger);
}

.workbench-statusbar__dot--connected {
  background: var(--st-success);
}

.workbench-statusbar__address {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.workbench-statusbar__switch {
  display: grid;
  width: 18px;
  height: 18px;
  flex: 0 0 auto;
  padding: 0;
  place-items: center;
  border: 0;
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.workbench-statusbar__switch :deep(svg) {
  width: 14px;
  height: 14px;
}

.workbench-statusbar__switch:hover {
  color: var(--st-text-primary);
}

.workbench-statusbar__switch:focus-visible {
  outline: 1px solid var(--st-border-focus);
  outline-offset: 1px;
}

.workbench-statusbar__version {
  flex: 0 0 auto;
}
</style>
