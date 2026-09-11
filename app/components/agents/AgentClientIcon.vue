<script setup lang="ts">
import type { AgentClient } from "~/stores/relay";
import { agentClientDefinitions } from "~/utils/agentClient";

const props = withDefaults(
  defineProps<{
    client: AgentClient;
    muted?: boolean;
    size?: number;
  }>(),
  {
    muted: false,
    size: 28,
  },
);

const definition = computed(() =>
  agentClientDefinitions.find((item) => item.client === props.client),
);
</script>

<template>
  <img
    v-if="definition"
    :src="definition.icon"
    :alt="definition.label"
    class="agent-client-icon"
    :class="{
      'agent-client-icon--monochrome': definition.monochrome,
      'agent-client-icon--muted': muted,
    }"
    :style="{ width: `${size}px`, height: `${size}px` }"
  />
</template>

<style scoped>
.agent-client-icon {
  display: block;
  object-fit: contain;
}

.agent-client-icon--monochrome {
  filter: var(--pr-monochrome-icon-filter);
}

.agent-client-icon--muted {
  filter: var(--pr-monochrome-icon-filter) grayscale(1);
  opacity: 0.45;
}
</style>
