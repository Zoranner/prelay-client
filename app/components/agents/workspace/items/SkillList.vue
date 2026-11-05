<script setup lang="ts">
defineOptions({ name: "AgentSkillList" });

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
</script>

<template>
  <div class="agent-items-view">
    <div v-if="items.length" class="agent-item-list">
      <AgentItemRow
        v-for="item in items"
        :key="item.name"
        :item="item"
        :primary="item.name"
        icon="ph:book-open-text"
        :pending="pending"
        @uninstall="emit('uninstall', $event)"
      >
        <template v-if="item.fileCount !== null" #facts>
          <span>{{ item.fileCount }} 个文件</span>
        </template>
        <template #detail>
          <span
            v-if="item.description"
            class="agent-detail-line agent-detail-line--ellipsis"
            :title="item.description"
          >
            {{ item.description }}
          </span>
          <span v-else class="agent-detail-line agent-detail-line--ellipsis">
            -
          </span>
        </template>
      </AgentItemRow>
    </div>
    <div v-else class="agent-empty-state">
      <EmptyState size="small" icon="ph:book-open-text" title="暂无 Skill" />
    </div>
  </div>
</template>

<style scoped src="./items.css"></style>
