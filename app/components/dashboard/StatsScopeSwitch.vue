<script setup lang="ts">
import type { StatsScope } from "~/stores/relay";

const model = defineModel<StatsScope>({ required: true });

const options: Array<{ label: string; value: StatsScope }> = [
  { label: "个人", value: "personal" },
  { label: "团队", value: "team" },
];
</script>

<template>
  <div class="stats-scope-switch" role="group" aria-label="统计视角">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="stats-scope-switch__option"
      :class="{
        'stats-scope-switch__option--active': model === option.value,
      }"
      :aria-pressed="model === option.value"
      @click="model = option.value"
    >
      {{ option.label }}
    </button>
  </div>
</template>

<style scoped>
/* 胶囊轨道对齐组件库 Tabs 的 pills 形态；段尺寸取 buttonSizes.tiny（h-5 / px-1.5 / text-xs），轨道内边距取 spacing token */
.stats-scope-switch {
  display: inline-flex;
  height: 24px;
  align-items: center;
  gap: var(--spacing-2xs);
  padding: var(--spacing-2xs);
  border-radius: 999px;
  background: var(--st-bg-surface);
}

.stats-scope-switch__option {
  height: 20px;
  padding: 0 6px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--st-text-secondary);
  font-size: 12px;
  line-height: 1;
  cursor: pointer;
  transition:
    background-color 150ms ease,
    color 150ms ease;
}

.stats-scope-switch__option:hover {
  background: var(--st-bg-elevated);
  color: var(--st-text-primary);
}

.stats-scope-switch__option--active,
.stats-scope-switch__option--active:hover {
  background: var(--st-primary);
  color: #fff;
  font-weight: 500;
}
</style>
