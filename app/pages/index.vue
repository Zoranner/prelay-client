<script setup lang="ts">
import type {
  ModelStats,
  ProviderStats,
  StatsOverview,
  StatsRange,
  TokenUsageTimelinePoint,
} from "~/stores/relay";
import { Button } from "@stellar/ui";
import StatsRangeSelect from "~/components/dashboard/StatsRangeSelect.vue";
import StatsBreakdownTable from "~/components/dashboard/StatsBreakdownTable.vue";
import StatsOverviewPanel from "~/components/dashboard/StatsOverview.vue";
import TokenUsageTrendChart from "~/components/dashboard/TokenUsageTrendChart.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

const { pending, invokeCommand } = useRelayCommand();
const overview = ref<StatsOverview | null>(null);
const models = ref<ModelStats[]>([]);
const providers = ref<ProviderStats[]>([]);
const timeline = ref<TokenUsageTimelinePoint[]>([]);
const selectedRange = ref<StatsRange>("this_week");

const modelRows = computed(() =>
  models.value.map((row, index) => ({
    id: `${row.model_requested ?? "unknown"}-${index}`,
    name: row.model_requested ?? "未识别模型",
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
    const [overviewValue, modelRows, providerRows, timelineRows] = await Promise.all([
      invokeCommand<StatsOverview>("stats_overview", range),
      invokeCommand<ModelStats[]>("stats_models", range),
      invokeCommand<ProviderStats[]>("stats_providers", range),
      invokeCommand<TokenUsageTimelinePoint[]>("stats_timeline", range),
    ]);
    overview.value = overviewValue;
    models.value = modelRows;
    providers.value = providerRows;
    timeline.value = timelineRows;
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

.dashboard-stat-lists {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-lg);
}

@media (max-width: 960px) {
  .dashboard-stat-lists {
    grid-template-columns: 1fr;
  }
}
</style>
