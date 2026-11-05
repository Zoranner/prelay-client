<script setup lang="ts">
defineOptions({ name: "AgentHookList" });

import { EmptyState } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import {
  agentItemKey,
  hasUnresolvedError,
  hookMatcher,
} from "~/utils/agent/itemDisplay";
import AgentItemRow from "~/components/agents/workspace/items/AgentItemRow.vue";

const props = defineProps<{
  items: AgentItem[];
  pending?: boolean;
}>();
const emit = defineEmits<{
  uninstall: [item: AgentItem];
}>();

const visibleItems = computed(() =>
  props.items.filter((item) => !hasUnresolvedError(item)),
);
const groups = computed(() => {
  const byEvent = new Map<string, AgentItem[]>();
  for (const item of visibleItems.value) {
    const handlers = byEvent.get(item.name) ?? [];
    handlers.push(item);
    byEvent.set(item.name, handlers);
  }
  return [...byEvent].map(([event, handlers]) => ({ event, handlers }));
});
</script>

<template>
  <div class="agent-items-view">
    <div v-if="visibleItems.length" class="agent-hook-list">
      <section
        v-for="group in groups"
        :key="group.event"
        class="agent-hook-group"
      >
        <header class="agent-hook-group__header">
          <span class="agent-hook-group__event" :title="group.event">
            {{ group.event }}
          </span>
          <span
            class="agent-hook-group__count"
            :aria-label="`${group.handlers.length} 个处理程序`"
          >
            {{ group.handlers.length }} 个
          </span>
        </header>
        <AgentItemRow
          v-for="item in group.handlers"
          :key="agentItemKey(item)"
          :item="item"
          :primary="hookMatcher(item) || '未限定'"
          icon="ph:webhooks-logo"
          :pending="pending"
          @uninstall="emit('uninstall', $event)"
        >
          <template #facts>
            <span class="agent-meta-badge">command</span>
          </template>
          <template v-if="item.command" #detail>
            <span
              class="agent-detail-line agent-detail-line--mono agent-detail-line--ellipsis"
              :title="item.command"
            >
              {{ item.command }}
            </span>
          </template>
        </AgentItemRow>
      </section>
    </div>
    <div v-else class="agent-empty-state">
      <EmptyState size="small" icon="ph:webhooks-logo" title="暂无 Hook" />
    </div>
  </div>
</template>

<style scoped src="./items.css"></style>
