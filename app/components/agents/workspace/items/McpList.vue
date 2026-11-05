<script setup lang="ts">
defineOptions({ name: "AgentMcpList" });

import { EmptyState } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import AgentItemRow from "~/components/agents/workspace/items/AgentItemRow.vue";

defineProps<{
  items: AgentItem[];
  pending?: boolean;
}>();
const emit = defineEmits<{
  uninstall: [item: AgentItem];
}>();

function transportLabel(transport: string | null) {
  return transport === "http" ? "HTTP · 远程" : "stdio · 本地进程";
}
</script>

<template>
  <div class="agent-items-view">
    <div v-if="items.length" class="agent-item-list">
      <AgentItemRow
        v-for="item in items"
        :key="item.name"
        :item="item"
        :primary="item.name"
        icon="ph:terminal-window"
        :pending="pending"
        @uninstall="emit('uninstall', $event)"
      >
        <template v-if="item.transport" #facts>
          <span class="agent-meta-badge">
            {{ transportLabel(item.transport) }}
          </span>
        </template>
        <template #detail>
          <span
            v-if="item.target"
            class="agent-detail-line agent-detail-line--mono agent-detail-line--ellipsis"
            :title="item.target"
          >
            {{ item.target }}
          </span>
          <span
            v-else
            class="agent-detail-line agent-detail-line--mono agent-detail-line--ellipsis"
          >
            -
          </span>
        </template>
      </AgentItemRow>
    </div>
    <div v-else class="agent-empty-state">
      <EmptyState size="small" icon="ph:terminal-window" title="暂无 MCP" />
    </div>
  </div>
</template>

<style scoped src="./items.css"></style>
