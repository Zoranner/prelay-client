<script setup lang="ts">
import { Tag } from "@stellar/ui";

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

function isRetired(model: ProviderModelToggle) {
  return model.state === "retired";
}

// 打开的模型用主色标签，没打开的用中性标签，目录里已没有的用错误色并带感叹号。
function colorProps(model: ProviderModelToggle) {
  if (isRetired(model)) return { semantic: "error" as const };
  return model.state === "enabled"
    ? { semantic: "primary" as const }
    : { palette: "gray" as const };
}

function toggleTitle(model: ProviderModelToggle) {
  if (isRetired(model)) return "目录里已没有这个模型，点击移除";
  return model.state === "enabled" ? "点击停用" : "点击启用";
}

function activate(model: ProviderModelToggle) {
  if (props.canToggle === false) return;
  if (isRetired(model)) {
    emit("remove", model.id);
    return;
  }
  emit("toggle", model.id);
}
</script>

<template>
  <div class="model-toggles">
    <button
      v-for="model in models"
      :key="model.id"
      type="button"
      class="model-toggles__chip"
      :title="toggleTitle(model)"
      :aria-pressed="isRetired(model) ? undefined : model.state === 'enabled'"
      :aria-label="
        isRetired(model) ? `移除目录已下架的模型 ${model.label}` : undefined
      "
      :disabled="canToggle === false"
      @click="activate(model)"
    >
      <Tag
        v-bind="colorProps(model)"
        size="small"
        :icon="isRetired(model) ? 'ph:warning-circle' : ''"
      >
        {{ model.label }}
      </Tag>
    </button>
  </div>
</template>

<style scoped>
.model-toggles {
  display: flex;
  min-width: 0;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}
.model-toggles__chip {
  padding: 0;
  border: 0;
  background: none;
  color: inherit;
  font: inherit;
  cursor: pointer;
}
.model-toggles__chip:hover:not(:disabled) {
  opacity: 0.85;
}
.model-toggles__chip:disabled {
  cursor: not-allowed;
}
</style>
