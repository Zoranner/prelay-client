<script setup lang="ts">
import {
  Button,
  DropdownMenu,
  Icon,
  MenuDivider,
  MenuItem,
} from "@stellar/ui";

import type { StatsRange } from "~/stores/relay";

type RangeOption = {
  label: string;
  value: StatsRange;
};

const props = defineProps<{ modelValue: StatsRange }>();
const emit = defineEmits<{ "update:modelValue": [value: StatsRange] }>();

const groups: Array<{ label: string; options: RangeOption[] }> = [
  {
    label: "按日",
    options: [
      { label: "今日", value: "today" },
      { label: "昨日", value: "yesterday" },
    ],
  },
  {
    label: "按周",
    options: [
      { label: "本周", value: "this_week" },
      { label: "上周", value: "last_week" },
    ],
  },
  {
    label: "按月",
    options: [
      { label: "本月", value: "this_month" },
      { label: "上月", value: "last_month" },
    ],
  },
  {
    label: "按年",
    options: [
      { label: "今年", value: "this_year" },
      { label: "去年", value: "last_year" },
    ],
  },
];

const selectedLabel = computed(() =>
  groups
    .flatMap((group) => group.options)
    .find((option) => option.value === props.modelValue)?.label ?? "本周",
);

function selectRange(value: StatsRange, close: () => void) {
  emit("update:modelValue", value);
  close();
}
</script>

<template>
  <DropdownMenu align="right">
    <template #trigger>
      <Button class="stats-range-trigger">
        <Icon icon="ph:calendar-blank" />
        <span>{{ selectedLabel }}</span>
        <Icon icon="ph:caret-down" />
      </Button>
    </template>
    <template #default="{ close }">
      <div class="stats-range-menu">
        <template v-for="(group, groupIndex) in groups" :key="group.label">
          <p class="stats-range-menu__group">{{ group.label }}</p>
          <MenuItem
            v-for="option in group.options"
            :key="option.value"
            :active="option.value === modelValue"
            :label="option.label"
            @click="selectRange(option.value, close)"
          />
          <MenuDivider v-if="groupIndex < groups.length - 1" />
        </template>
      </div>
    </template>
  </DropdownMenu>
</template>

<style scoped>
.stats-range-trigger {
  gap: var(--spacing-xs);
}

.stats-range-menu {
  min-width: 168px;
  padding: var(--spacing-xs) 0;
}

.stats-range-menu__group {
  margin: var(--spacing-xs) var(--spacing-md);
  color: var(--st-text-muted);
  font-size: 12px;
}
</style>
