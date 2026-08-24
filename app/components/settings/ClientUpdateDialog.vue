<script setup lang="ts">
import { Button, Modal } from "stellar-ui";

const visible = defineModel<boolean>("visible", { default: false });
const version = defineModel<string | null>("version", { default: null });
const clientUpdate = useClientUpdate();

async function install() {
  await clientUpdate.install();
}
</script>

<template>
  <Modal
    v-model:visible="visible"
    title="更新已准备就绪"
    size="small"
    :show-cancel="false"
    :show-confirm="false"
  >
    <p class="client-update-dialog__copy">
      Prelay {{ version }} 已下载。开始安装后，当前应用将退出。
    </p>

    <template #footer>
      <Button :disabled="clientUpdate.installing.value" @click="visible = false">
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
