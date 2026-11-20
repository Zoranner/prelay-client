<script setup lang="ts">
defineOptions({ name: "AgentWorkspaceSidebar" });

import { Icon, List, ListItem } from "@stellar/ui";
import type { AgentClient } from "~/types/relay";
import AgentClientIcon from "~/components/agents/client/Icon.vue";

type AgentClientCard = {
  client: AgentClient;
  installed: boolean;
  label: string;
  version: string;
};

defineProps<{
  activeClient: AgentClient;
  clients: AgentClientCard[];
  statusLoading: boolean;
}>();

const emit = defineEmits<{
  selectClient: [client: AgentClient];
}>();
</script>

<template>
  <aside class="agent-sidebar">
    <List class="agent-client-list" :divided="false">
      <ListItem
        v-for="client in clients"
        :key="client.client"
        :active="activeClient === client.client"
        clickable
        @click="emit('selectClient', client.client)"
      >
        <template #prefix>
          <span class="agent-client-icon-frame">
            <AgentClientIcon
              :client="client.client"
              :muted="!client.installed || statusLoading"
            />
            <span v-if="statusLoading" class="agent-client-loading">
              <Icon icon="ph:circle-notch" size="28" />
            </span>
          </span>
        </template>
        <span class="agent-client-identity">
          <span>{{ client.label }}</span>
          <small>{{ client.version }}</small>
        </span>
      </ListItem>
    </List>
  </aside>
</template>

<style scoped>
.agent-sidebar {
  display: grid;
  min-height: 0;
  grid-template-rows: minmax(0, 1fr);
  padding-right: var(--spacing-md);
  border-right: 1px solid var(--st-border-divider);
}

.agent-client-list {
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
}

.agent-client-list :deep(.st-list-item > div:first-child) {
  display: flex;
  width: 40px;
  height: 40px;
  align-items: center;
  justify-content: center;
}

.agent-client-icon-frame {
  display: grid;
  position: relative;
  width: 40px;
  height: 40px;
  place-items: center;
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-md);
  background: var(--st-bg-elevated);
}

.agent-client-loading {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  pointer-events: none;
  color: var(--st-text-primary);
  animation: agent-sidebar-loading-spin 800ms linear infinite;
}

.agent-client-identity {
  display: grid;
  gap: 2px;
}

.agent-client-identity small {
  color: var(--st-text-secondary);
  font-size: 12px;
}

@keyframes agent-sidebar-loading-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
