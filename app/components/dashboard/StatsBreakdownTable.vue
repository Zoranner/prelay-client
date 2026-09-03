<script setup lang="ts">
import { Table } from "@stellar/ui";
import { modelCatalogLabel } from "~/utils/modelCatalog";

type StatsBreakdownRow = Record<string, unknown> & {
  id: string;
  name: string;
  model_requested?: string | null;
  model_requested_display_name?: string | null;
  total_requests: number;
  successful_requests: number;
  input_tokens: number;
  output_tokens: number;
  average_latency_ms: number | null;
};

defineProps<{
  emptyMessage: string;
  rows: StatsBreakdownRow[];
  title: string;
}>();

const columns = [
  { key: "name", title: "名称", width: 176, ellipsis: true },
  { key: "total_requests", title: "请求", width: 76, align: "right" as const },
  { key: "success_rate", title: "成功率", width: 88, align: "right" as const },
  { key: "input_tokens", title: "输入", width: 96, align: "right" as const },
  { key: "output_tokens", title: "输出", width: 96, align: "right" as const },
  {
    key: "average_latency_ms",
    title: "平均响应",
    width: 108,
    align: "right" as const,
  },
];

function formatTokens(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toLocaleString("zh-CN");
}

function formatLatency(value: number | null) {
  if (value === null) return "-";
  return value >= 1_000
    ? `${(value / 1_000).toFixed(2)} s`
    : `${Math.round(value)} ms`;
}

function successRate(row: StatsBreakdownRow) {
  if (!row.total_requests) return "-";
  return `${((row.successful_requests / row.total_requests) * 100).toFixed(1)}%`;
}

function rowName(row: StatsBreakdownRow) {
  if (row.model_requested !== undefined) {
    return (
      row.model_requested_display_name?.trim() ||
      modelCatalogLabel(row.model_requested) ||
      row.model_requested ||
      row.name
    );
  }
  return row.name;
}
</script>

<template>
  <section class="stats-breakdown-table">
    <header class="stats-breakdown-table__header">
      <h2>{{ title }}</h2>
      <span>{{ rows.length }} 项</span>
    </header>
    <Table
      class="stats-breakdown-table__grid"
      :columns="columns"
      :data="rows"
      :empty-text="emptyMessage"
      fixed-header
      row-key="id"
    >
      <template #cell-name="{ row }">
        <span :title="rowName(row)">{{ rowName(row) }}</span>
      </template>
      <template #cell-success_rate="{ row }">
        {{ successRate(row) }}
      </template>
      <template #cell-input_tokens="{ row }">
        {{ formatTokens(row.input_tokens) }}
      </template>
      <template #cell-output_tokens="{ row }">
        {{ formatTokens(row.output_tokens) }}
      </template>
      <template #cell-average_latency_ms="{ row }">
        {{ formatLatency(row.average_latency_ms) }}
      </template>
    </Table>
  </section>
</template>

<style scoped>
.stats-breakdown-table {
  display: flex;
  height: 272px;
  min-width: 0;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.stats-breakdown-table__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.stats-breakdown-table__header h2,
.stats-breakdown-table__header span {
  margin: 0;
}

.stats-breakdown-table__header h2 {
  color: var(--st-text-primary);
  font-size: 15px;
}

.stats-breakdown-table__header span {
  color: var(--st-text-secondary);
  font-size: 12px;
}

.stats-breakdown-table__grid {
  min-height: 0;
  flex: 1;
}
</style>
