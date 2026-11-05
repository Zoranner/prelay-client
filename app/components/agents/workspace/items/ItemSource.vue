<script setup lang="ts">
import { Tag } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import {
  agentSourceLabel,
  agentSourceSemantic,
  hasUnresolvedError,
} from "~/utils/agent/itemDisplay";

defineProps<{
  item: AgentItem;
}>();
</script>

<template>
  <span v-if="hasUnresolvedError(item)" class="agent-item-source-empty">-</span>
  <Tag v-else :semantic="agentSourceSemantic(item.source)" size="small">
    {{ agentSourceLabel(item.source) }}
  </Tag>
</template>

<style scoped>
.agent-item-source-empty {
  color: var(--st-text-muted);
}
</style>
