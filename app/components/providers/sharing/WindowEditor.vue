<script setup lang="ts">
import { Checkbox, Input, Select, Toggle } from "@stellar/ui";
import type { EditableWindow } from "./types";

defineProps<{
  windows: EditableWindow[];
  disabled?: boolean;
}>();

const weekdayOptions = [
  { label: "周一", value: 1 },
  { label: "周二", value: 2 },
  { label: "周三", value: 3 },
  { label: "周四", value: 4 },
  { label: "周五", value: 5 },
  { label: "周六", value: 6 },
  { label: "周日", value: 7 },
];
const windowLabels: Record<EditableWindow["kind"], string> = {
  day: "日",
  week: "周",
  month: "月",
};
</script>

<template>
  <div class="window-editor">
    <div
      v-for="window in windows"
      :key="window.kind"
      class="window-row"
      :class="{ 'window-row--disabled': !window.enabled }"
    >
      <Checkbox
        :model-value="window.enabled"
        :label="windowLabels[window.kind]"
        :disabled="disabled"
        @update:model-value="window.enabled = $event === true"
      />
      <Input
        v-model="window.limit"
        type="number"
        size="small"
        placeholder="Token 额度"
        :disabled="disabled || !window.enabled"
      />
      <Toggle
        v-model="window.accumulate"
        size="small"
        label="累计"
        :disabled="disabled || !window.enabled"
      />
      <Input
        v-model="window.reset_time"
        type="time"
        size="small"
        :disabled="disabled || !window.enabled"
      />
      <Select
        v-if="window.kind === 'week'"
        v-model="window.reset_weekday"
        :options="weekdayOptions"
        :disabled="disabled || !window.enabled"
      />
      <Input
        v-if="window.kind === 'month'"
        v-model="window.reset_day"
        type="number"
        size="small"
        :min="1"
        :max="28"
        placeholder="日"
        :disabled="disabled || !window.enabled"
      />
    </div>
  </div>
</template>

<style scoped>
.window-editor {
  display: grid;
  gap: var(--spacing-xs);
}

.window-row {
  display: grid;
  grid-template-columns: 64px minmax(140px, 1fr) auto 112px repeat(2, auto);
  gap: var(--spacing-xs);
  align-items: center;
}

.window-row--disabled {
  opacity: 0.55;
}

@media (max-width: 760px) {
  .window-row {
    grid-template-columns: 64px minmax(120px, 1fr) auto;
  }
}
</style>
