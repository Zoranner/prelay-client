<script setup lang="ts">
defineOptions({ name: "AgentPluginList" });

import { EmptyState, Icon, TagGroup } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import {
  agentItemKindIcon,
  agentItemKindLabel,
} from "~/utils/agent/itemDisplay";
import AgentItemRow from "~/components/agents/workspace/items/AgentItemRow.vue";

defineProps<{
  items: AgentItem[];
  pending?: boolean;
}>();
const emit = defineEmits<{
  uninstall: [item: AgentItem];
}>();

function artifactSummary(item: AgentItem) {
  return item.artifacts
    .map((group) => `${agentItemKindLabel(group.kind)} ${group.items.length}`)
    .join(" · ");
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
        icon="ph:puzzle-piece"
        :pending="pending"
        @uninstall="emit('uninstall', $event)"
      >
        <template v-if="item.version || item.artifacts.length" #facts>
          <span v-if="item.version" class="agent-meta-badge">
            {{ item.version }}
          </span>
          <span v-if="item.artifacts.length">{{ artifactSummary(item) }}</span>
        </template>
        <template v-if="item.artifacts.length" #detail>
          <div class="agent-artifact-lines">
            <div
              v-for="group in item.artifacts"
              :key="group.kind"
              class="agent-artifact-line"
            >
              <span class="agent-artifact-kind">
                <Icon :icon="agentItemKindIcon(group.kind)" size="14" />
                {{ agentItemKindLabel(group.kind) }}
              </span>
              <TagGroup :items="group.items" />
            </div>
          </div>
        </template>
      </AgentItemRow>
    </div>
    <div v-else class="agent-empty-state">
      <EmptyState size="small" icon="ph:puzzle-piece" title="暂无插件" />
    </div>
  </div>
</template>

<style scoped src="./items.css"></style>
