<script setup lang="ts">
import type { StatsOverview as StatsOverviewData } from "~/stores/relay";

import { StatCard } from "stellar-ui";

const props = defineProps<{ overview: StatsOverviewData | null }>();

function formatTokens(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toLocaleString("zh-CN");
}

function formatLatency(value: number | null | undefined) {
  if (value === null || value === undefined) return "-";
  return value >= 1_000 ? `${(value / 1_000).toFixed(2)} s` : `${value} ms`;
}

const cacheHitRate = computed(() => {
  const inputTokens = props.overview?.input_tokens ?? 0;
  const cacheReadTokens = props.overview?.cache_read_tokens ?? 0;
  const totalInputTokens = inputTokens + cacheReadTokens;
  if (!totalInputTokens) return "-";
  return `${((cacheReadTokens / totalInputTokens) * 100).toFixed(1)}%`;
});

const metrics = computed(() => [
  {
    title: "请求数",
    value: props.overview?.total_requests ?? 0,
    icon: "ph:calendar-check",
  },
  {
    title: "平均响应",
    value: formatLatency(props.overview?.average_latency_ms),
    icon: "ph:timer",
  },
  {
    title: "总 Token",
    value: formatTokens(
      (props.overview?.input_tokens ?? 0) + (props.overview?.output_tokens ?? 0),
    ),
    icon: "ph:chart-line-up",
  },
  {
    title: "输入 Token",
    value: formatTokens(props.overview?.input_tokens ?? 0),
    icon: "ph:clock-counter-clockwise",
  },
  {
    title: "输出 Token",
    value: formatTokens(props.overview?.output_tokens ?? 0),
    icon: "ph:coins",
  },
  {
    title: "缓存命中率",
    value: cacheHitRate.value,
    icon: "ph:database",
  },
]);
</script>

<template>
  <section class="dashboard-stats">
    <StatCard
      v-for="metric in metrics"
      :key="metric.title"
      :icon="metric.icon"
      :title="metric.title"
      :value="metric.value"
    />
  </section>
</template>

<style scoped>
.dashboard-stats {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: var(--spacing-md);
}

@media (max-width: 1180px) {
  .dashboard-stats {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .dashboard-stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
