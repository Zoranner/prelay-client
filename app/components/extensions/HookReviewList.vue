<script setup lang="ts">
import type { HookReviewItem } from "~/stores/relay";

defineProps<{
  review: HookReviewItem[];
  files?: string[];
}>();
</script>

<template>
  <section class="hook-review">
    <h3>需要在 Codex 中审核并信任 Hook</h3>
    <p class="hook-review__notice">
      配置写入本机后不会自动生效；Codex 审核并信任前，Hook 不会运行。
    </p>
    <div v-if="files?.length" class="hook-review__files">
      <h4>将写入的 Hook 支持文件</h4>
      <ul>
        <li v-for="file in files" :key="file">
          <code>{{ file }}</code>
        </li>
      </ul>
    </div>
    <ul class="hook-review__items">
      <li
        v-for="item in review"
        :key="`${item.event}-${item.matcher}-${item.command}`"
      >
        <strong>
          {{ item.event }}
          <template v-if="item.matcher"> · {{ item.matcher }}</template>
        </strong>
        <code>command: {{ item.command }}</code>
        <code v-if="item.commandWindows"
          >commandWindows: {{ item.commandWindows }}</code
        >
      </li>
    </ul>
  </section>
</template>

<style scoped>
.hook-review {
  min-height: 0;
  overflow-y: auto;
  padding: var(--spacing-md);
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-md);
  background: var(--st-bg-surface);
}

.hook-review h3 {
  margin: 0 0 var(--spacing-sm);
  font-size: var(--text-sm);
}

.hook-review__notice {
  margin: 0;
  color: var(--st-text-muted);
  font-size: var(--text-xs);
}

.hook-review__files h4 {
  margin: 0 0 var(--spacing-xs);
  color: var(--st-text-secondary);
  font-size: var(--text-xs);
  font-weight: 600;
}

.hook-review ul {
  display: grid;
  gap: var(--spacing-sm);
  margin: 0;
  padding: 0;
  list-style: none;
}

.hook-review li {
  display: grid;
  gap: 4px;
}

.hook-review code {
  overflow-wrap: anywhere;
  color: var(--st-text-secondary);
  font-family: var(--font-family-mono);
  font-size: var(--text-xs);
}
</style>
