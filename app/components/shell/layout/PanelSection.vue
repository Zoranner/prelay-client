<script setup lang="ts">
defineProps<{
  title: string;
}>();
</script>

<template>
  <section class="panel-section">
    <header class="panel-section-header">
      <div class="panel-section-heading">
        <h2 class="panel-section-title">{{ title }}</h2>
        <slot name="header-inline" />
      </div>
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
  /* 与 text-base 配套的行高：标题盒与 24px 的行内控件同高，居中不靠微调 */
  line-height: 24px;
}

/* 标题与紧随其后的行内控件成组，右侧工具条仍然靠右 */
.panel-section-heading {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-md);
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
