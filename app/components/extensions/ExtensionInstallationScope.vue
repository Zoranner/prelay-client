<script setup lang="ts">
import type { ExtensionCatalogPackage } from "~/stores/relay";
import AgentClientAvatarGroup from "~/components/agents/AgentClientAvatarGroup.vue";

const props = defineProps<{
  package: ExtensionCatalogPackage;
}>();

</script>

<template>
  <AgentClientAvatarGroup
    v-if="package.installedClients.length"
    :clients="package.installedClients"
  />
  <span v-else class="extension-installation-empty">-</span>
</template>

<style scoped>
.extension-installation-empty {
  color: var(--st-text-muted);
}
</style>
