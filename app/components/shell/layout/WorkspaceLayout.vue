<script setup lang="ts">
withDefaults(
  defineProps<{
    ready?: boolean;
  }>(),
  {
    ready: true,
  },
);
</script>

<template>
  <div class="workspace-main">
    <div v-if="ready === false" class="workspace-status">
      <slot name="status" />
    </div>
    <template v-else>
      <div class="workspace-toolbar">
        <slot name="toolbar" />
      </div>
      <div class="workspace-results">
        <slot />
      </div>
    </template>
  </div>
</template>

<style scoped>
.workspace-main {
  display: flex;
  position: relative;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  padding-left: var(--spacing-lg);
}

.workspace-status {
  display: grid;
  flex: 1;
  place-content: center;
  justify-items: center;
}

.workspace-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 0 0 var(--spacing-md);
}

.workspace-toolbar :deep(.workspace-toolbar__actions) {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: var(--spacing-sm);
}

.workspace-toolbar :deep(.workspace-update-hint) {
  white-space: nowrap;
}

.workspace-results {
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
</style>
