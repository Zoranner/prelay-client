<script setup lang="ts">
import { Button } from "@stellar/ui";

defineProps<{
  provider: string;
  source: string;
  model: string;
  canMoveUp: boolean;
  canMoveDown: boolean;
}>();

const emit = defineEmits<{
  move: [delta: number];
  remove: [];
}>();
</script>

<template>
  <div class="model-row">
    <small>{{ provider }} {{ source }} / {{ model }}</small>
    <div class="model-row__actions">
      <Button
        square
        size="small"
        variant="ghost"
        icon="ph:arrow-up"
        aria-label="上移供应商优先级"
        title="上移供应商优先级"
        :disabled="!canMoveUp"
        @click="emit('move', -1)"
      />
      <Button
        square
        size="small"
        variant="ghost"
        icon="ph:arrow-down"
        aria-label="下移供应商优先级"
        title="下移供应商优先级"
        :disabled="!canMoveDown"
        @click="emit('move', 1)"
      />
      <Button
        square
        size="small"
        semantic="error"
        variant="solid"
        icon="ph:trash"
        aria-label="删除供应商映射"
        title="删除供应商映射"
        @click="emit('remove')"
      />
    </div>
  </div>
</template>

<style scoped>
.model-row small {
  color: var(--st-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-row__actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
</style>
