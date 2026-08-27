<script setup lang="ts">
defineProps<{
  title: string;
}>();
</script>

<template>
  <section class="panel-section">
    <header class="panel-section-header">
      <h2 class="panel-section-title">{{ title }}</h2>
      <div class="panel-section-actions">
        <slot name="header-actions" />
      </div>
    </header>

    <div class="panel-section-content">
      <slot />
    </div>

    <slot name="footer" />
  </section>
</template>

<style scoped>
.panel-section {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
}

.panel-section-header {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
  min-height: 48px;
  padding: 0 var(--spacing-lg);
  border-bottom: 1px solid var(--st-border-divider);
}

.panel-section-title {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 16px;
  font-weight: 650;
}

.panel-section-content {
  display: flex;
  flex: 1;
  min-height: 0;
  flex-direction: column;
  padding: var(--spacing-lg);
  overflow: hidden;
}

.panel-section-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
</style>
