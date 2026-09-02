<script setup lang="ts">
import { Avatar, Card } from "@stellar/ui";
import { Avatar as DiceBearAvatar, Style } from "@dicebear/core";
import cutouts from "@dicebear/styles/cutouts.json";

import type { UserLeaderboardEntry } from "~/stores/relay";

type UserLeaderboardRow = UserLeaderboardEntry & Record<string, unknown>;

defineProps<{
  rows: UserLeaderboardRow[];
}>();

const cutoutsStyle = new Style(cutouts);

function avatarSrc(row: UserLeaderboardEntry) {
  return new DiceBearAvatar(cutoutsStyle, {
    seed: row.identity_id,
  }).toDataUri();
}

function formatTokens(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toLocaleString("zh-CN");
}
</script>

<template>
  <Card full-height class="user-leaderboard-card" :hoverable="false">
    <section class="user-leaderboard">
      <header class="user-leaderboard__header">
        <h2>用户排行榜</h2>
        <span>总 Token</span>
      </header>
      <ol class="user-leaderboard__list">
        <li
          v-for="row in rows"
          :key="row.identity_id"
          class="user-leaderboard__item"
        >
          <Avatar
            class="leaderboard-avatar"
            :src="avatarSrc(row)"
            :alt="row.display_name"
            size="large"
            shape="circle"
          />
          <div class="leaderboard-entry">
            <span class="leaderboard-name" :title="row.display_name">
              {{ row.display_name }}
            </span>
            <strong class="leaderboard-tokens">
              {{ formatTokens(row.total_tokens) }}
            </strong>
          </div>
          <span class="leaderboard-rank">{{ row.rank }}</span>
        </li>
      </ol>
    </section>
  </Card>
</template>

<style scoped>
.user-leaderboard-card {
  height: 400px;
  min-width: 0;
  overflow: hidden;
}

.user-leaderboard {
  display: flex;
  height: 100%;
  min-width: 0;
  flex-direction: column;
  gap: var(--spacing-lg);
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

.user-leaderboard__list {
  display: grid;
  min-height: 0;
  flex: 1;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  list-style: none;
  gap: var(--spacing-md);
  align-content: start;
}

.user-leaderboard__item {
  display: grid;
  grid-template-columns: 40px minmax(0, 1fr) 24px;
  column-gap: var(--spacing-sm);
  align-items: start;
  min-width: 0;
}

.leaderboard-rank {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  justify-self: end;
  align-self: center;
  color: var(--st-text-primary);
  background: var(--st-bg-surface);
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-sm);
  font-family: var(--font-family-mono);
  font-size: 12px;
  font-weight: 600;
  line-height: 1;
  text-align: right;
}

.leaderboard-avatar {
  width: 40px;
  height: 40px;
}

.leaderboard-entry {
  display: grid;
  min-width: 0;
  gap: 2px;
  padding-top: 2px;
  align-content: start;
  justify-items: start;
}

.leaderboard-name,
.leaderboard-tokens {
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.leaderboard-name {
  color: var(--st-text-primary);
}

.leaderboard-tokens {
  justify-self: start;
  color: var(--st-text-secondary);
  font-family: var(--font-family-mono);
  font-size: 12px;
  font-weight: 500;
  text-align: left;
}
</style>
