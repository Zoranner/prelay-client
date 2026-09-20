<script setup lang="ts">
import { Avatar, Card, Skeleton } from "@stellar/ui";

import type { UserLeaderboardEntry } from "~/stores/relay";
import { identityAvatarSrc } from "~/utils/identityAvatar";
import { formatTokens } from "~/utils/tokenFormat";

type UserLeaderboardRow = UserLeaderboardEntry & Record<string, unknown>;

const props = defineProps<{
  rows: UserLeaderboardRow[];
  loading?: boolean;
}>();
</script>

<template>
  <Card class="user-leaderboard-card" :hoverable="false">
    <section class="user-leaderboard">
      <header class="user-leaderboard__header">
        <h2>用户排行榜</h2>
        <span>总 Token</span>
      </header>
      <Skeleton
        v-if="props.loading && !props.rows.length"
        :rows="6"
        avatar
        avatar-size="large"
        animated
      />
      <ol v-else class="user-leaderboard__list">
        <li
          v-for="row in rows"
          :key="row.identity_id"
          class="user-leaderboard__item"
        >
          <Avatar
            class="leaderboard-avatar"
            :src="identityAvatarSrc(row.identity_id)"
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
  min-width: 0;
}

.user-leaderboard {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: var(--spacing-md);
  container-type: inline-size;
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
  min-width: 0;
  overflow: hidden;
  color: var(--st-text-primary);
  font-size: 15px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-leaderboard__header span {
  flex: 0 0 auto;
  color: var(--st-text-secondary);
  font-size: 12px;
  white-space: nowrap;
}

.user-leaderboard__list {
  display: grid;
  margin: 0;
  padding: 0;
  list-style: none;
  gap: var(--spacing-md);
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

/* 卡片变窄时按容器宽度收紧，不换行、不横向滚动。 */
@container (max-width: 260px) {
  .user-leaderboard__item {
    grid-template-columns: 40px minmax(0, 1fr) 20px;
    column-gap: var(--spacing-xs);
  }

  .leaderboard-rank {
    width: 20px;
    height: 20px;
    font-size: 11px;
  }
}

@container (max-width: 200px) {
  .user-leaderboard__item {
    grid-template-columns: 40px minmax(0, 1fr) 18px;
  }

  .leaderboard-rank {
    width: 18px;
    height: 18px;
    font-size: 10px;
  }

  .user-leaderboard__header h2 {
    font-size: 14px;
  }

  /* 标题行只留标题，右侧的总量说明让位，避免折行。 */
  .user-leaderboard__header span {
    display: none;
  }
}
</style>
