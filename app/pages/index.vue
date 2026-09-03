<script setup lang="ts">
import type {
  ModelStats,
  ProviderStats,
  StatsOverview,
  StatsRange,
  TokenUsageTimelinePoint,
  UserLeaderboardEntry,
} from "~/stores/relay";
import { Button } from "@stellar/ui";
import StatsRangeSelect from "~/components/dashboard/StatsRangeSelect.vue";
import StatsBreakdownTable from "~/components/dashboard/StatsBreakdownTable.vue";
import StatsOverviewPanel from "~/components/dashboard/StatsOverview.vue";
import TokenUsageTrendChart from "~/components/dashboard/TokenUsageTrendChart.vue";
import PanelSection from "~/components/shell/PanelSection.vue";
import UserLeaderboardTable from "~/components/dashboard/UserLeaderboardTable.vue";
import { modelCatalogLabel } from "~/utils/modelCatalog";

const { pending, invokeCommand } = useRelayCommand();
const overview = ref<StatsOverview | null>(null);
const models = ref<ModelStats[]>([]);
const providers = ref<ProviderStats[]>([]);
const timeline = ref<TokenUsageTimelinePoint[]>([]);
const leaderboard = ref<UserLeaderboardEntry[]>([]);
const leaderboardRows = computed(() =>
  leaderboard.value.map((row) => ({ ...row })),
);
const selectedRange = ref<StatsRange>("this_week");

const modelRows = computed(() =>
  models.value.map((row, index) => ({
    id: `${row.model_requested ?? "unknown"}-${index}`,
    name:
      row.model_requested_display_name?.trim() ||
      modelCatalogLabel(row.model_requested) ||
      row.model_requested ||
      "未识别模型",
    model_requested: row.model_requested,
    model_requested_display_name: row.model_requested_display_name,
    total_requests: row.total_requests,
    successful_requests: row.successful_requests,
    input_tokens: row.input_tokens,
    output_tokens: row.output_tokens,
    average_latency_ms: row.average_latency_ms,
  })),
);
const providerRows = computed(() =>
  providers.value.map((row, index) => ({
    id: row.provider_id ?? `unknown-${index}`,
    name: row.provider_name ?? "未识别供应商",
    total_requests: row.total_requests,
    successful_requests: row.successful_requests,
    input_tokens: row.input_tokens,
    output_tokens: row.output_tokens,
    average_latency_ms: row.average_latency_ms,
  })),
);

async function loadDashboard() {
  try {
    const range = { range: selectedRange.value };
    const [
      overviewValue,
      modelRows,
      providerRows,
      timelineRows,
      leaderboardRows,
    ] = await Promise.all([
      invokeCommand<StatsOverview>("stats_overview", range),
      invokeCommand<ModelStats[]>("stats_models", range),
      invokeCommand<ProviderStats[]>("stats_providers", range),
      invokeCommand<TokenUsageTimelinePoint[]>("stats_timeline", range),
      invokeCommand<UserLeaderboardEntry[]>("stats_leaderboard", {
        range: selectedRange.value,
        metric: "total_tokens",
        limit: 50,
      }),
    ]);
    overview.value = overviewValue;
    models.value = modelRows;
    providers.value = providerRows;
    timeline.value = timelineRows;
    leaderboard.value = leaderboardRows;
  } catch {
    // The command composable exposes the stable error to this view.
  }
}

onMounted(loadDashboard);
watch(selectedRange, loadDashboard);
</script>

<template>
  <main class="page-dashboard">
    <PanelSection title="仪表盘">
      <template #header-actions>
        <StatsRangeSelect v-model="selectedRange" />
        <Button
          variant="primary"
          icon="ph:arrows-clockwise"
          :disabled="pending"
          @click="loadDashboard"
        >
          {{ pending ? "刷新中..." : "刷新" }}
        </Button>
      </template>
      <div class="dashboard-content">
        <StatsOverviewPanel :overview="overview" />
        <div class="dashboard-primary-grid">
          <TokenUsageTrendChart :points="timeline" :range="selectedRange" />
          <UserLeaderboardTable :rows="leaderboardRows" />
        </div>
        <div class="dashboard-stat-lists">
          <StatsBreakdownTable
            empty-message="暂无供应商统计。"
            :rows="providerRows"
            title="供应商统计"
          />
          <StatsBreakdownTable
            empty-message="暂无模型统计。"
            :rows="modelRows"
            title="模型统计"
          />
        </div>
      </div>
    </PanelSection>
  </main>
</template>

<style scoped>
.page-dashboard {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--pr-dashboard-padding);
}

.dashboard-content {
  display: grid;
  min-height: 0;
  flex: 1;
  gap: var(--spacing-lg);
  overflow: auto;
}

.dashboard-primary-grid {
  display: grid;
  min-width: 0;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: var(--spacing-lg);
}

.dashboard-primary-grid > :first-child {
  grid-column: span 5;
}

.dashboard-primary-grid > :last-child {
  grid-column: span 1;
}

.dashboard-stat-lists {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-lg);
}

@media (max-width: 1180px) {
  .dashboard-primary-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .dashboard-primary-grid > :first-child {
    grid-column: span 2;
  }

  .dashboard-stat-lists {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .dashboard-primary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .dashboard-primary-grid > :first-child,
  .dashboard-primary-grid > :last-child {
    grid-column: span 1;
  }
}

@media (max-width: 560px) {
  .dashboard-primary-grid {
    grid-template-columns: 1fr;
  }
}
</style>
