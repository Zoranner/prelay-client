<script setup lang="ts">
import { Button, Toggle } from "@stellar/ui";

export type ProviderModelState = "enabled" | "available" | "retired";

export type ProviderModelToggle = {
  id: string;
  label: string;
  state: ProviderModelState;
};

const props = defineProps<{
  models: ProviderModelToggle[];
  canToggle?: boolean;
}>();

const emit = defineEmits<{
  toggle: [modelId: string];
  remove: [modelId: string];
}>();

function isOn(model: ProviderModelToggle) {
  return model.state === "enabled";
}

// 目录已下架的模型只能移除，不能再启停。
function isLocked(model: ProviderModelToggle) {
  return props.canToggle === false || model.state === "retired";
}

function toggleTitle(model: ProviderModelToggle) {
  if (model.state === "retired") return "目录里已没有这个模型，只能移除";
  return isOn(model) ? "点击停用" : "点击启用";
}
</script>

<template>
  <div class="model-toggles">
    <div
      v-for="model in models"
      :key="model.id"
      class="model-toggles__row"
      :class="`model-toggles__row--${model.state}`"
    >
      <span class="model-toggles__name" :title="model.label">
        {{ model.label }}
      </span>
      <span :title="toggleTitle(model)">
        <Toggle
          :model-value="isOn(model)"
          :semantic="model.state === 'retired' ? 'warning' : 'primary'"
          :disabled="isLocked(model)"
          :aria-label="model.label"
          @update:model-value="emit('toggle', model.id)"
        />
      </span>
      <Button
        v-if="model.state === 'retired'"
        square
        size="small"
        semantic="error"
        variant="solid"
        icon="ph:trash"
        aria-label="移除目录已下架的模型"
        title="目录里已没有这个模型，保存前必须移除"
        :disabled="canToggle === false"
        @click="emit('remove', model.id)"
      />
    </div>
  </div>
</template>

<style scoped>
.model-toggles {
  display: grid;
  gap: var(--spacing-xs);
}
.model-toggles__row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: var(--spacing-md);
  min-height: 32px;
}
.model-toggles__name {
  min-width: 0;
  overflow: hidden;
  color: var(--st-text-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
}
.model-toggles__row--available .model-toggles__name {
  color: var(--st-text-muted);
}
.model-toggles__row--retired .model-toggles__name {
  color: var(--st-warning);
}
</style>
