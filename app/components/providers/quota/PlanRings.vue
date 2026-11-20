<script setup lang="ts">
defineOptions({ name: "QuotaPlanRings" });

import type { QuotaWindow } from "~/types/quota";

const props = withDefaults(
  defineProps<{
    windows: QuotaWindow[];
    size?: number;
  }>(),
  { size: 24 },
);

const radii = [4.5, 7.5, 10.5];

function slotFor(window: QuotaWindow) {
  if (window.span_secs != null && window.span_secs > 0) {
    if (window.span_secs <= 5 * 60 * 60) return 0;
    if (window.span_secs <= 7 * 24 * 60 * 60) return 1;
    return 2;
  }
  const label = window.label.replace(/\s+/g, "");
  if (label.includes("小时") || label === "日") return 0;
  if (label.includes("周") || label.includes("天")) return 1;
  return 2;
}

const rings = computed(() => {
  const slots: Array<QuotaWindow | undefined> = [];
  for (const window of props.windows) {
    const index = slotFor(window);
    slots[index] ??= window;
  }
  return radii.map((radius, index) => {
    const window = slots[index];
    if (!window) {
      return {
        key: `disabled-${index}`,
        radius,
        percent: 0,
        disabled: true,
        color: "var(--st-text-muted)",
      };
    }
    const percent = clampPercent(100 - window.used_percent);
    return {
      key: `${window.label}-${index}`,
      radius,
      percent,
      disabled: false,
      color: colorFor(percent),
    };
  });
});

const tightest = computed(() =>
  Math.min(
    100,
    ...props.windows.map((window) => clampPercent(100 - window.used_percent)),
  ),
);

function clampPercent(value: number) {
  return Math.max(0, Math.min(100, Number.isFinite(value) ? value : 0));
}

function colorFor(percent: number) {
  if (percent <= 15) return "var(--st-danger)";
  if (percent <= 35) return "var(--st-warning)";
  return "var(--st-primary)";
}
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    class="quota-rings"
    :aria-label="
      windows.length ? `剩余 ${Math.round(tightest)}%` : '无可用额度窗口'
    "
  >
    <template v-for="ring in rings" :key="ring.key">
      <circle
        class="quota-rings__track"
        :class="{ 'quota-rings__track--disabled': ring.disabled }"
        cx="12"
        cy="12"
        :r="ring.radius"
        fill="none"
      />
      <circle
        v-if="!ring.disabled"
        class="quota-rings__value"
        cx="12"
        cy="12"
        :r="ring.radius"
        fill="none"
        pathLength="100"
        :stroke="ring.color"
        :stroke-dasharray="`${ring.percent} 100`"
        transform="rotate(-90 12 12)"
      />
    </template>
  </svg>
</template>

<style scoped>
.quota-rings {
  flex: 0 0 auto;
}

.quota-rings__track {
  stroke: var(--st-border-divider);
  stroke-width: 2;
}

.quota-rings__track--disabled {
  stroke: var(--st-text-muted);
  opacity: 0.45;
  stroke-dasharray: 2 2;
}

.quota-rings__value {
  stroke-linecap: round;
  stroke-width: 2;
}
</style>
