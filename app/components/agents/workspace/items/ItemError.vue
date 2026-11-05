<script setup lang="ts">
import { Icon, Tooltip } from "@stellar/ui";

defineProps<{
  message: string | null;
}>();
</script>

<template>
  <span v-if="message" class="agent-item-error">
    <Tooltip :content="message" position="top">
      <span
        class="agent-item-error__trigger"
        role="img"
        tabindex="0"
        :aria-label="message"
      >
        <Icon icon="ph:warning-circle" aria-hidden="true" />
      </span>
    </Tooltip>
  </span>
</template>

<style scoped>
.agent-item-error {
  display: inline-flex;
  flex: 0 0 14px;
  width: 14px;
  height: 14px;
  align-items: center;
  justify-content: center;
  color: var(--st-danger);
  line-height: 0;
  cursor: help;
}

.agent-item-error > :deep(*) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 0;
}

.agent-item-error__trigger {
  display: inline-flex;
  width: 14px;
  height: 14px;
  align-items: center;
  justify-content: center;
  line-height: 0;
}

.agent-item-error :deep(svg) {
  display: block;
  width: 14px;
  height: 14px;
}

.agent-item-error:focus-within {
  outline: 1px solid var(--st-border-active);
  outline-offset: 2px;
}
</style>
