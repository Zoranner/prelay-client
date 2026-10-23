<script setup lang="ts">
import type { HookReviewItem } from "~/stores/relay";

defineProps<{
  review: HookReviewItem[];
}>();
</script>

<template>
  <section class="hook-review">
    <h3>需要在 Codex 中审核并信任 Hook</h3>
    <ul>
      <li
        v-for="item in review"
        :key="`${item.event}-${item.matcher}-${item.command}`"
      >
        <strong>
          {{ item.event }}
          <template v-if="item.matcher"> · {{ item.matcher }}</template>
        </strong>
        <code>{{ item.commandWindows || item.command }}</code>
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
