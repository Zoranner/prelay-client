<script setup lang="ts">
defineOptions({ name: "ExtensionInstallationScope" });

import type { ExtensionCatalogPackage } from "~/types/relay";
import AgentClientAvatarGroup from "~/components/agents/client/AvatarGroup.vue";

defineProps<{
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
