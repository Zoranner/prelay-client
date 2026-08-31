<script setup lang="ts">
import { Button, Modal } from "@stellar/ui";

const visible = defineModel<boolean>("visible", { default: false });
const version = defineModel<string | null>("version", { default: null });
const clientUpdate = useClientUpdate();
const workspaceExit = useWorkspaceExitGuard();
let exitRegistration: ReturnType<typeof workspaceExit.register> | undefined;

function closeImmediately() {
  visible.value = false;
}

function requestClose() {
  if (exitRegistration) void exitRegistration.requestExit();
  else closeImmediately();
}

async function install() {
  await clientUpdate.install();
}

watch(visible, (isVisible) => {
  if (!isVisible) {
    exitRegistration?.unregister();
    exitRegistration = undefined;
    return;
  }
  exitRegistration = workspaceExit.register({
    close: closeImmediately,
    state: () => (clientUpdate.installing.value ? "blocked" : "allow"),
  });
});

onBeforeUnmount(() => exitRegistration?.unregister());
</script>

<template>
  <Modal
    :visible="visible"
    title="更新已准备就绪"
    size="small"
    :blocked="clientUpdate.installing.value"
    :show-cancel="false"
    :show-confirm="false"
    @update:visible="
      (nextVisible) => (nextVisible ? (visible = true) : requestClose())
    "
  >
    <p class="client-update-dialog__copy">
      Prelay
      {{ version }}
      已下载，安装时将短暂退出，不影响其他智能体和正在进行的对话及调用。
    </p>

    <template #footer>
      <Button :disabled="clientUpdate.installing.value" @click="requestClose">
        稍后安装
      </Button>
      <Button
        variant="primary"
        :loading="clientUpdate.installing.value"
        @click="install"
      >
        开始安装
      </Button>
    </template>
  </Modal>
</template>

<style scoped>
.client-update-dialog__copy {
  margin: 0;
  color: var(--st-text-secondary);
  font-size: 14px;
  line-height: 1.6;
}
</style>
