<script setup lang="ts">
import type { ProviderModelOption } from "~/utils/providerTemplates";

const props = defineProps<{
  options: ProviderModelOption[];
  disabledModels: string[];
  canToggle?: boolean;
}>();

const emit = defineEmits<{ toggle: [modelId: string] }>();

function isDisabled(modelId: string) {
  return props.disabledModels.includes(modelId);
}
</script>

<template>
  <div class="model-toggles">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="model-toggle"
      :class="{ 'model-toggle--off': isDisabled(option.value) }"
      :aria-pressed="!isDisabled(option.value)"
      :title="isDisabled(option.value) ? '点击启用' : '点击禁用'"
      :disabled="canToggle === false"
      @click="emit('toggle', option.value)"
    >
      <span class="model-toggle__name">{{ option.label }}</span>
      <span class="model-toggle__state">
        {{ isDisabled(option.value) ? "已禁用" : "已启用" }}
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
.model-toggle--off {
  border-style: dashed;
  background: transparent;
}
.model-toggle--off .model-toggle__name {
  color: var(--st-text-muted);
  text-decoration: line-through;
}
</style>
