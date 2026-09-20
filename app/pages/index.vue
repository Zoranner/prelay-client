<script setup lang="ts">
import type {
  ModelStats,
  ProviderStats,
  StatsScope,
  StatsOverview,
  StatsRange,
  TokenUsageTimelinePoint,
  UserLeaderboardEntry,
} from "~/stores/relay";
import { Button } from "@stellar/ui";
import StatsRangeSelect from "~/components/dashboard/StatsRangeSelect.vue";
import StatsBreakdownTable from "~/components/dashboard/StatsBreakdownTable.vue";
import ModelLeaderboard from "~/components/dashboard/ModelLeaderboard.vue";
import StatsOverviewPanel from "~/components/dashboard/StatsOverview.vue";
import StatsScopeSwitch from "~/components/dashboard/StatsScopeSwitch.vue";
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
const statsScope = ref<StatsScope>("personal");

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
    // 视角（个人 / 团队）对所有可切卡片生效；用户排行榜天生全站，不带 scope
    const query = { range: selectedRange.value, scope: statsScope.value };
    const [
      overviewValue,
      modelRows,
      providerRows,
      timelineRows,
      leaderboardRows,
    ] = await Promise.all([
      invokeCommand<StatsOverview>("stats_overview", query),
      invokeCommand<ModelStats[]>("stats_models", query),
      invokeCommand<ProviderStats[]>("stats_providers", query),
      invokeCommand<TokenUsageTimelinePoint[]>("stats_timeline", query),
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
watch([selectedRange, statsScope], loadDashboard);
</script>

<template>
  <main class="page-dashboard">
    <PanelSection title="仪表盘">
      <template #header-inline>
        <StatsScopeSwitch v-model="statsScope" />
      </template>
      <template #header-actions>
        <StatsRangeSelect
          v-model="selectedRange"
          class="dashboard-header-control"
        />
        <Button
          class="dashboard-header-control"
          semantic="primary"
          variant="solid"
          icon="ph:arrows-clockwise"
          :disabled="pending"
          @click="loadDashboard"
        >
          {{ pending ? "刷新中..." : "刷新" }}
        </Button>
      </template>
      <div class="dashboard-content">
        <div class="dashboard-main">
          <StatsOverviewPanel :overview="overview" />
          <TokenUsageTrendChart :points="timeline" :range="selectedRange" />
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
        <div class="dashboard-rail">
          <UserLeaderboardTable :rows="leaderboardRows" :loading="pending" />
          <ModelLeaderboard :rows="models" :loading="pending" />
        </div>
      </div>
    </PanelSection>
  </main>
</template>

<style scoped>
/* 表头控件保持固有宽度：RadioGroup 根节点自带 w-full，这里覆盖成自适应宽度 */
.dashboard-header-control {
  flex: 0 0 auto;
  width: auto;
}

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
  /* 对齐系统：主列弹性，右栏随窗口在 260~320px 内收缩 */
  grid-template-columns: minmax(0, 1fr) clamp(260px, 24vw, 320px);
  align-items: start;
}

.dashboard-main {
  display: grid;
  min-width: 0;
  gap: var(--spacing-lg);
}

.dashboard-rail {
  display: grid;
  min-width: 0;
  align-content: start;
  gap: var(--spacing-lg);
}

.dashboard-stat-lists {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-lg);
}

@media (max-width: 1180px) {
  .dashboard-stat-lists {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 900px) {
  /* 放不下两栏时右栏落到内容下方，而不是把卡片压窄 */
  .dashboard-content {
    grid-template-columns: 1fr;
  }
}
</style>
