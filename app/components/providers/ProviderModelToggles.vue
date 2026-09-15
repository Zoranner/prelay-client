<script setup lang="ts">
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

function stateLabel(state: ProviderModelState) {
  if (state === "enabled") return "已启用";
  if (state === "available") return "未启用";
  return "目录已下架";
}

function stateTitle(state: ProviderModelState) {
  if (state === "enabled") return "点击停用";
  if (state === "available") return "点击启用";
  return "目录条目里已没有这个模型，保存前必须移除";
}

function activate(model: ProviderModelToggle) {
  if (props.canToggle === false) return;
  if (model.state === "retired") {
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
      class="model-toggle"
      :class="`model-toggle--${model.state}`"
      :aria-pressed="model.state === 'enabled'"
      :title="stateTitle(model.state)"
      :disabled="canToggle === false"
      @click="activate(model)"
    >
      <span class="model-toggle__name">{{ model.label }}</span>
      <span class="model-toggle__state">
        {{ stateLabel(model.state) }}
      </span>
    </button>
  </div>
</template>

<style scoped>
.model-toggles {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}
.model-toggle {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-xs);
  min-width: 0;
  padding: 2px var(--spacing-sm);
  border: 1px solid var(--st-border);
  border-radius: var(--radius-sm);
  background: var(--st-bg-elevated);
  color: var(--st-text-primary);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}
.model-toggle:hover:not(:disabled) {
  border-color: var(--st-border-active);
}
.model-toggle:disabled {
  cursor: default;
}
.model-toggle__name {
  min-width: 0;
  overflow-wrap: anywhere;
}
.model-toggle__state {
  color: var(--st-text-muted);
  font-size: 11px;
  white-space: nowrap;
}
.model-toggle--available {
  border-style: dashed;
  background: transparent;
}
.model-toggle--available .model-toggle__name {
  color: var(--st-text-muted);
}
.model-toggle--retired {
  border-style: dashed;
  border-color: var(--st-warning);
  background: transparent;
}
.model-toggle--retired .model-toggle__name {
  color: var(--st-text-muted);
  text-decoration: line-through;
}
.model-toggle--retired .model-toggle__state {
  color: var(--st-warning);
}
</style>
