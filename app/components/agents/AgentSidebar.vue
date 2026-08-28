<script setup lang="ts">
import { Icon, List, ListItem } from "@stellar/ui";
import type { AgentClient } from "~/stores/relay";

type AgentClientCard = {
  client: AgentClient;
  icon: string;
  installed: boolean;
  label: string;
  monochrome: boolean;
  version: string;
};

defineProps<{
  activeWorkspace: AgentClient | "extensions";
  clients: AgentClientCard[];
  statusLoading: boolean;
}>();

const emit = defineEmits<{
  selectClient: [client: AgentClient];
  selectExtensions: [];
}>();
</script>

<template>
  <aside class="agent-sidebar">
    <List class="agent-client-list" :divided="false">
      <ListItem
        v-for="client in clients"
        :key="client.client"
        :active="activeWorkspace === client.client"
        clickable
        @click="emit('selectClient', client.client)"
      >
        <template #prefix>
          <span class="agent-client-icon-frame">
            <img
              :src="client.icon"
              :alt="client.label"
              class="agent-client-icon"
              :class="{
                'agent-client-icon--monochrome': client.monochrome,
                'agent-client-icon--uninstalled': !client.installed,
                'agent-client-icon--loading': statusLoading,
              }"
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
    <List class="agent-extension-library" :divided="false">
      <ListItem
        :active="activeWorkspace === 'extensions'"
        clickable
        @click="emit('selectExtensions')"
      >
        <template #prefix>
          <span class="agent-client-icon-frame">
            <Icon icon="ph:storefront" size="24" />
          </span>
        </template>
        <span class="agent-client-identity"><span>扩展库</span></span>
      </ListItem>
    </List>
  </aside>
</template>

<style scoped>
.agent-sidebar {
  display: grid;
  min-height: 0;
  grid-template-rows: minmax(0, 1fr) auto;
  padding-right: var(--spacing-md);
  border-right: 1px solid var(--st-border-divider);
}

.agent-client-list {
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
}

.agent-client-list :deep(.st-list-item > div:first-child),
.agent-extension-library :deep(.st-list-item > div:first-child) {
  display: flex;
  width: 40px;
  height: 40px;
  align-items: center;
  justify-content: center;
}

.agent-extension-library {
  margin-top: 0;
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--st-border-divider);
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

.agent-client-icon {
  display: block;
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.agent-client-icon--monochrome {
  filter: var(--pr-monochrome-icon-filter);
}

.agent-client-icon--uninstalled,
.agent-client-icon--loading {
  filter: var(--pr-monochrome-icon-filter) grayscale(1);
  opacity: 0.45;
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
