<script setup lang="ts">
import type { StatsOverview } from "~/stores/relay";

import { Card } from "stellar-ui";

const props = defineProps<{ overview: StatsOverview | null }>();

const successRate = computed(() => {
  if (!props.overview?.total_requests) return 0;
  return (props.overview.successful_requests / props.overview.total_requests) * 100;
});
</script>

<template>
  <Card>
    <section class="dashboard-panel">
      <div class="dashboard-panel__header">
        <div>
          <h2 class="dashboard-panel__title">请求结果</h2>
          <p class="dashboard-panel__description">成功与失败请求的占比</p>
        </div>
        <strong class="dashboard-panel__total">
          {{ overview?.total_requests ?? 0 }}
        </strong>
      </div>
      <div class="outcome-meter">
        <div
          class="outcome-meter__fill"
          :style="{ width: `${successRate}%` }"
        />
      </div>
      <div class="outcome-summary">
        <p class="outcome-summary__item">
          成功 <strong class="outcome-summary__value">{{ overview?.successful_requests ?? 0 }}</strong>
        </p>
        <p class="outcome-summary__item">
          失败 <strong class="outcome-summary__value outcome-summary__value--danger">{{ overview?.failed_requests ?? 0 }}</strong>
        </p>
        <p class="outcome-summary__item">
          成功率 <strong class="outcome-summary__value">{{ successRate.toFixed(1) }}%</strong>
        </p>
      </div>
    </section>
  </Card>
</template>

<style scoped>
.dashboard-panel {
  display: grid;
  gap: var(--spacing-md);
}

.dashboard-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.dashboard-panel__title,
.dashboard-panel__description,
.outcome-summary__item {
  margin: 0;
}

.dashboard-panel__title {
  color: var(--st-text-primary);
  font-size: 15px;
}

.dashboard-panel__description,
.outcome-summary__item {
  color: var(--st-text-secondary);
}

.dashboard-panel__total,
.outcome-summary__value {
  color: var(--st-text-primary);
}

.dashboard-panel__total {
  font-size: 24px;
}

.outcome-meter,
.ranking-bar {
  overflow: hidden;
  background: var(--st-bg-elevated);
}

.outcome-meter {
  height: 10px;
  border-radius: 4px;
}

.outcome-meter__fill {
  height: 100%;
  background: var(--st-primary);
}

.outcome-summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--spacing-md);
}

.outcome-summary__value {
  margin-left: var(--spacing-xs);
}

.outcome-summary__value--danger {
  color: var(--st-danger);
}
</style>
