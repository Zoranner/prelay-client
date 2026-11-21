<script setup lang="ts">
defineOptions({ name: "ProviderQuotaRings" });

import type { RingSlot } from "~/utils/provider/quota/view";

const props = withDefaults(
  defineProps<{
    slots: RingSlot[];
    size?: number;
  }>(),
  { size: 24 },
);

const radii = computed(() =>
  props.slots.length >= 3 ? [4.5, 7.5, 10.5] : [6, 9.5],
);

const rings = computed(() =>
  radii.value.map((radius, index) => {
    const slot = props.slots[index];
    if (!slot?.configured) {
      return {
        key: slot?.key ?? `empty-${index}`,
        radius,
        percent: 0,
        configured: false,
        color: "var(--st-text-muted)",
      };
    }
    return {
      key: slot.key,
      radius,
      percent: slot.percent,
      configured: true,
      color: colorFor(slot.percent),
    };
  }),
);

const tightest = computed(() => {
  const configured = props.slots.filter((slot) => slot.configured);
  if (!configured.length) return 100;
  return Math.min(...configured.map((slot) => slot.percent));
});

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
    <template v-for="ring in rings" :key="ring.key">
      <circle
        class="quota-rings__track"
        :class="{ 'quota-rings__track--empty': !ring.configured }"
        cx="12"
        cy="12"
        :r="ring.radius"
        fill="none"
      />
      <circle
        v-if="ring.configured"
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

.quota-rings__track--empty {
  stroke: var(--st-text-muted);
  stroke-dasharray: 2 2;
  opacity: 0.45;
}

.quota-rings__value {
  stroke-linecap: round;
  stroke-width: 2;
}
</style>
