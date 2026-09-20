<script setup lang="ts">
import { Card, EmptyState, Skeleton } from "@stellar/ui";

import type { ModelStats } from "~/stores/relay";
import { modelCatalogLabel } from "~/utils/modelCatalog";
import { formatTokens } from "~/utils/tokenFormat";

const props = withDefaults(
  defineProps<{
    rows: ModelStats[];
    limit?: number;
    loading?: boolean;
  }>(),
  { limit: 5, loading: false },
);

const entries = computed(() =>
  props.rows
    .map((row) => ({
      id: row.model_requested ?? "unknown",
      name:
        row.model_requested_display_name?.trim() ||
        modelCatalogLabel(row.model_requested) ||
        "未识别模型",
      tokens: (row.input_tokens ?? 0) + (row.output_tokens ?? 0),
    }))
    .filter((entry) => entry.tokens > 0)
    .sort(
      (left, right) =>
        right.tokens - left.tokens || left.name.localeCompare(right.name),
    )
    .slice(0, props.limit),
);

const peak = computed(() => entries.value[0]?.tokens ?? 0);

function barWidth(tokens: number) {
  if (!peak.value) return "0%";
  return `${Math.max(6, Math.round((tokens / peak.value) * 100))}%`;
}
</script>

<template>
  <Card :hoverable="false">
    <section class="model-leaderboard">
      <header class="model-leaderboard__header">
        <h2>模型排行榜</h2>
        <span>按 Token</span>
      </header>
      <Skeleton v-if="props.loading && !entries.length" :rows="5" animated />
      <EmptyState
        v-else-if="!entries.length"
        size="small"
        title="暂无模型数据"
      />
      <ol v-else class="model-leaderboard__list">
        <li
          v-for="entry in entries"
          :key="entry.id"
          class="model-leaderboard__item"
        >
          <span class="model-leaderboard__name" :title="entry.name">
            {{ entry.name }}
          </span>
          <span class="model-leaderboard__value">
            {{ formatTokens(entry.tokens) }}
          </span>
          <span class="model-leaderboard__track">
            <i :style="{ width: barWidth(entry.tokens) }" />
          </span>
        </li>
      </ol>
    </section>
  </Card>
</template>

<style scoped>
.model-leaderboard {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: var(--spacing-md);
}

.model-leaderboard__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.model-leaderboard__header h2 {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 15px;
}

.model-leaderboard__header span {
  color: var(--st-text-secondary);
  font-size: 12px;
  white-space: nowrap;
}

.model-leaderboard__list {
  display: grid;
  gap: var(--spacing-sm);
  margin: 0;
  padding: 0;
  list-style: none;
}

.model-leaderboard__item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 2px var(--spacing-sm);
  min-width: 0;
}

.model-leaderboard__name {
  min-width: 0;
  overflow: hidden;
  color: var(--st-text-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-leaderboard__value {
  color: var(--st-text-secondary);
  font-family: var(--font-family-mono);
  font-size: 12px;
  white-space: nowrap;
}

.model-leaderboard__track {
  display: block;
  grid-column: 1 / -1;
  height: 4px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--st-bg-surface);
}

.model-leaderboard__track i {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: var(--st-primary);
}
</style>
