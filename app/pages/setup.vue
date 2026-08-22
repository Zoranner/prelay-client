<script setup lang="ts">
import { Button, Card, Input, useNotification } from "stellar-ui";

const settings = useRelaySettings();
const { pending } = useRelayCommand();
const notifications = useNotification();
const relayUrl = ref("");
const route = useRoute();
const isChangingAddress = computed(() => route.query.change === "1");

function returnToWorkspace() {
  void navigateTo("/");
}

async function save() {
  try {
    await settings.connect(relayUrl.value);
    notifications.success("管理服务已连接");
    await navigateTo("/");
  } catch {
    // The command composable exposes the stable error to this view.
  }
}
</script>

<template>
  <main class="setup-screen">
    <Button
      v-if="isChangingAddress"
      class="setup-screen__back"
      square
      size="small"
      variant="ghost"
      icon="ph:arrow-left"
      aria-label="返回"
      title="返回"
      @click="returnToWorkspace"
    />
    <Card class="setup-card" :hoverable="false">
      <form class="setup-form" @submit.prevent="save">
        <p class="setup-form__eyebrow">连接管理服务</p>
        <p>
          输入部署的服务地址。供应商配置、接入点与请求记录将按当前 Windows
          身份保存在该服务中。
        </p>
        <Input
          v-model.trim="relayUrl"
          autocomplete="url"
          label="服务地址"
          placeholder="https://relay.example.com"
          required
          type="url"
        />
        <Button variant="primary" :disabled="pending" type="submit">
          {{ pending ? "正在连接..." : "继续" }}
        </Button>
      </form>
    </Card>
  </main>
</template>

<style scoped>
.setup-screen {
  position: relative;
  display: grid;
  min-height: 0;
  place-items: center;
  overflow: auto;
  padding: var(--spacing-xl);
}

.setup-screen__back {
  position: absolute;
  top: var(--spacing-md);
  left: var(--spacing-md);
}

.setup-card {
  width: min(100%, 448px);
}

.setup-form {
  display: grid;
  gap: var(--spacing-lg);
}

.setup-form p {
  margin: 0;
}

.setup-form__eyebrow {
  color: var(--st-primary);
  font-weight: 650;
}

.setup-form p {
  color: var(--st-text-secondary);
}
</style>
