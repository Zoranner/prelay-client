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

const orderedWindows = ["five_hour", "week", "month"] as const;
const radii = [4.5, 7.5, 10.5];

const rings = computed(() => {
  const byKind = new Map(
    props.windows.map((window) => [window.window, window]),
  );
  return orderedWindows.flatMap((kind, index) => {
    const window = byKind.get(kind);
    if (!window) return [];
    const percent = clampPercent(window.remaining_percent);
    return [{ kind, radius: radii[index], percent, color: colorFor(percent) }];
  });
});

const tightest = computed(() =>
  Math.min(
    100,
    ...props.windows.map((window) => clampPercent(window.remaining_percent)),
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
    :aria-label="`剩余 ${Math.round(tightest)}%`"
  >
    <circle
      v-for="ring in rings"
      :key="`track-${ring.kind}`"
      class="quota-rings__track"
      cx="12"
      cy="12"
      :r="ring.radius"
      fill="none"
    />
    <circle
      v-for="ring in rings"
      :key="`value-${ring.kind}`"
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

.quota-rings__value {
  stroke-linecap: round;
  stroke-width: 2;
}
</style>
