<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { Button, useConfirm } from "@stellar/ui";

const props = defineProps<{
  relayUrl: string | null;
}>();

const managementApi = useRelayManagementApiStatus();
const clientUpdate = useClientUpdate();
const workspaceExit = useWorkspaceExitGuard();
const { isOpen: isConfirmOpen } = useConfirm();
const clientVersion = ref<string | null>(null);
const isSwitchingRelayAddress = ref(false);
const isConnected = computed(
  () => Boolean(props.relayUrl) && !managementApi.error.value,
);
const statusTitle = computed(() =>
  isConnected.value ? "服务地址已连接" : "服务地址不可用",
);
const updateState = computed(() => clientUpdate.state.value);
const updateIcon = computed(() => {
  switch (updateState.value) {
    case "checking":
      return "ph:spinner-gap";
    case "available":
      return "ph:download-simple";
    case "downloading":
      return "ph:circle-notch";
    case "ready":
      return "ph:arrow-square-in";
    default:
      return "";
  }
});
const updateTitle = computed(() => {
  switch (updateState.value) {
    case "checking":
      return "正在检查更新";
    case "available":
      return `下载 Prelay ${clientUpdate.version.value}`;
    case "downloading":
      return "正在下载更新";
    case "ready":
      return `安装 Prelay ${clientUpdate.version.value}`;
    default:
      return "检查更新";
  }
});
const isUpdatePending = computed(
  () => updateState.value === "checking" || updateState.value === "downloading",
);

async function switchRelayAddress() {
  if (isSwitchingRelayAddress.value || isConfirmOpen.value) return;

  isSwitchingRelayAddress.value = true;
  const canExit = await workspaceExit.requestExit({
    confirmText: "放弃并切换",
    message: "切换服务地址会丢弃当前未保存的修改。",
  });
  isSwitchingRelayAddress.value = false;
  if (!canExit) return;

  managementApi.clear();
  void navigateTo("/setup?change=1");
}

function checkForUpdate() {
  if (updateState.value === "idle") void clientUpdate.check();
}

function handleUpdateAction() {
  if (updateState.value === "available") void clientUpdate.download();
  if (updateState.value === "ready") clientUpdate.openInstallDialog();
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
  <footer class="dashboard-statusbar">
    <div class="dashboard-statusbar__connection">
      <span
        class="dashboard-statusbar__dot"
        :class="{ 'dashboard-statusbar__dot--connected': isConnected }"
        :title="statusTitle"
      />
      <span
        class="dashboard-statusbar__address"
        :title="relayUrl ?? '未配置服务地址'"
      >
        {{ relayUrl ?? "未配置服务地址" }}
      </span>
      <Button
        square
        size="tiny"
        variant="text"
        class="dashboard-statusbar__switch-relay"
        icon="ph:arrows-left-right"
        aria-label="切换服务地址"
        title="切换服务地址"
        :disabled="isSwitchingRelayAddress || isConfirmOpen"
        @click="switchRelayAddress"
      />
    </div>
    <div class="dashboard-statusbar__update">
      <button
        class="dashboard-statusbar__version"
        type="button"
        :title="updateTitle"
        :disabled="isUpdatePending"
        @click="checkForUpdate"
      >
        v{{ clientVersion ?? "-" }}
      </button>
      <Button
        v-if="updateIcon"
        square
        size="tiny"
        variant="text"
        :class="{
          'dashboard-statusbar__update-action--spinning': isUpdatePending,
        }"
        :icon="updateIcon"
        :disabled="isUpdatePending"
        :aria-label="updateTitle"
        :title="updateTitle"
        @click="handleUpdateAction"
      />
    </div>
  </footer>
</template>

<style scoped>
.dashboard-statusbar {
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

.dashboard-statusbar__connection {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-sm);
}

.dashboard-statusbar__dot {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  border-radius: 999px;
  background: var(--st-danger);
}

.dashboard-statusbar__dot--connected {
  background: var(--st-success);
}

.dashboard-statusbar__address {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-statusbar__switch-relay {
  margin-left: calc(var(--spacing-xs) - var(--spacing-sm));
}

.dashboard-statusbar__update {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: var(--spacing-xs);
}

.dashboard-statusbar__version {
  padding: 0;
  border: 0;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font: inherit;
}

.dashboard-statusbar__version:hover:not(:disabled) {
  color: var(--st-text-primary);
}

.dashboard-statusbar__version:focus-visible {
  outline: 1px solid var(--st-border-active);
  outline-offset: 2px;
}

.dashboard-statusbar__version:disabled {
  cursor: default;
}

.dashboard-statusbar__update-action--spinning :deep(svg) {
  animation: dashboard-statusbar-spin 0.8s linear infinite;
}

@keyframes dashboard-statusbar-spin {
  to {
    transform: rotate(1turn);
  }
}
</style>
