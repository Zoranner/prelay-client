<script setup lang="ts">
import { AvatarGroup, type AvatarGroupItem } from "@stellar/ui";
import type { AgentClient } from "~/stores/relay";
import { agentClientDefinitions } from "~/utils/agentClient";

const props = defineProps<{
  clients: AgentClient[];
}>();

const cell = computed(() => {
  const clients = props.clients
    .map((client) =>
      agentClientDefinitions.find((definition) => definition.client === client),
    )
    .filter((definition): definition is (typeof agentClientDefinitions)[number] =>
      Boolean(definition),
    );
  const items: AvatarGroupItem[] = clients.map((client) => ({
    src: client.icon,
    alt: client.label,
  }));
  return {
    items,
    title: `已安装到：${clients.map((client) => client.label).join("、")}`,
  };
});
</script>

<template>
  <AvatarGroup
    class="agent-client-avatar-group"
    :items="cell.items"
    :max="4"
    size="small"
    shape="circle"
    variant="capsule"
    :aria-label="cell.title"
    :title="cell.title"
  />
</template>

<style scoped>
.agent-client-avatar-group :deep(img) {
  filter: var(--pr-monochrome-icon-filter);
}
</style>
