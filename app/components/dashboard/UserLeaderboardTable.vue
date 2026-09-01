<script setup lang="ts">
import { Table } from "@stellar/ui";

import type { UserLeaderboardEntry } from "~/stores/relay";

type UserLeaderboardRow = UserLeaderboardEntry & Record<string, unknown>;

defineProps<{
  rows: UserLeaderboardRow[];
}>();

const columns = [
  { key: "rank", title: "排名", width: 64, align: "right" as const },
  { key: "display_name", title: "用户", width: 176, ellipsis: true },
  { key: "activity_count", title: "活动", width: 76, align: "right" as const },
  { key: "total_tokens", title: "Token", width: 96, align: "right" as const },
  { key: "success_rate", title: "成功率", width: 88, align: "right" as const },
];

function leaderboardRow(row: unknown) {
  return row as UserLeaderboardEntry;
}

function formatTokens(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toLocaleString("zh-CN");
}

function formatRate(value: number) {
  return `${(value * 100).toFixed(1)}%`;
}
</script>

<template>
  <section class="user-leaderboard">
    <header class="user-leaderboard__header">
      <h2>用户排行榜</h2>
      <span>{{ rows.length }} 人</span>
    </header>
    <Table
      class="user-leaderboard__grid"
      :columns="columns"
      :data="rows"
      empty-text="暂无用户活动"
      fixed-header
      row-key="identity_id"
    >
      <template #cell-display_name="{ row }">
        <span :title="leaderboardRow(row).display_name">
          {{ leaderboardRow(row).display_name }}
        </span>
      </template>
      <template #cell-total_tokens="{ row }">
        {{ formatTokens(leaderboardRow(row).total_tokens) }}
      </template>
      <template #cell-success_rate="{ row }">
        {{ formatRate(leaderboardRow(row).success_rate) }}
      </template>
    </Table>
  </section>
</template>

<style scoped>
.user-leaderboard {
  display: flex;
  height: 272px;
  min-width: 0;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.user-leaderboard__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.user-leaderboard__header h2,
.user-leaderboard__header span {
  margin: 0;
}

.user-leaderboard__header h2 {
  color: var(--st-text-primary);
  font-size: 15px;
}

.user-leaderboard__header span {
  color: var(--st-text-secondary);
  font-size: 12px;
}

.user-leaderboard__grid {
  min-height: 0;
  flex: 1;
}
</style>
