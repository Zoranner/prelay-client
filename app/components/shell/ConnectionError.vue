<script setup lang="ts">
import { Button, Card, Result } from "@stellar/ui";

defineOptions({ name: "ConnectionError" });

defineProps<{
  relayUrl: string | null;
}>();

const emit = defineEmits<{
  switchAddress: [];
  reload: [];
}>();
</script>

<template>
  <Card class="connection-error" :hoverable="false">
    <Result
      semantic="error"
      title="无法连接服务地址"
      description="请确认服务正在运行，或切换服务地址后重试。"
    >
      <div class="connection-error__address">
        <span>服务地址</span>
        <code :title="relayUrl ?? '未配置服务地址'">
          {{ relayUrl ?? "未配置服务地址" }}
        </code>
      </div>
      <div class="connection-error__actions">
        <Button icon="ph:arrows-left-right" @click="emit('switchAddress')">
          切换服务地址
        </Button>
        <Button
          semantic="primary"
          variant="solid"
          icon="ph:arrows-clockwise"
          @click="emit('reload')"
        >
          重新加载
        </Button>
      </div>
    </Result>
  </Card>
</template>

<style scoped>
.connection-error {
  width: min(100%, 480px);
}

.connection-error__address {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: baseline;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-sm);
  background: var(--st-bg-surface);
  text-align: left;
}

.connection-error__address span {
  color: var(--st-text-muted);
  font-size: 12px;
  white-space: nowrap;
}

.connection-error__address code {
  min-width: 0;
  overflow: hidden;
  color: var(--st-text-primary);
  font-family: var(--font-family-mono);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.connection-error__actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-md);
}
</style>
