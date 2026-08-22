<script setup lang="ts">
import * as echarts from "echarts";
import { Card } from "stellar-ui";

import type { StatsRange, TokenUsageTimelinePoint } from "~/stores/relay";
import { parseTimelineBucket } from "~/utils/stats";

const props = defineProps<{
  points: TokenUsageTimelinePoint[];
  range: StatsRange;
}>();

const chartElement = ref<HTMLElement | null>(null);
let chart: echarts.ECharts | undefined;
let resizeObserver: ResizeObserver | undefined;

function formatBucket(bucket: string) {
  const date = parseTimelineBucket(bucket);
  if (!date) return bucket;

  if (props.range === "today" || props.range === "yesterday") {
    return new Intl.DateTimeFormat("zh-CN", { hour: "2-digit" }).format(date);
  }
  if (["this_year", "last_year", "all"].includes(props.range)) {
    return new Intl.DateTimeFormat("zh-CN", {
      year: "numeric",
      month: "numeric",
    }).format(date);
  }
  return new Intl.DateTimeFormat("zh-CN", {
    month: "numeric",
    day: "numeric",
  }).format(date);
}

const categories = computed(() => props.points.map((point) => formatBucket(point.bucket)));
const cacheHitRates = computed(() =>
  props.points.map((point) => {
    const totalInput = point.input_tokens + point.cache_read_tokens;
    return totalInput ? Number(((point.cache_read_tokens / totalInput) * 100).toFixed(1)) : 0;
  }),
);

function renderChart() {
  if (!chart) return;
  chart.setOption({
    color: ["#38bdf8", "#34d399", "#fbbf24", "#a78bfa", "#fb7185"],
    grid: { top: 62, right: 54, bottom: 40, left: 54 },
    legend: {
      top: 0,
      left: "center",
      itemWidth: 22,
      itemHeight: 12,
      itemGap: 20,
      textStyle: { color: "#cbd5e1", fontSize: 14 },
    },
    tooltip: {
      trigger: "axis",
      backgroundColor: "#1a1f2e",
      borderColor: "#334155",
      borderWidth: 1,
      padding: [10, 12],
      textStyle: { color: "#f1f5f9" },
    },
    xAxis: {
      type: "category",
      data: categories.value,
      boundaryGap: true,
      axisLine: { lineStyle: { color: "#475569" } },
      axisTick: { show: false, alignWithLabel: true },
      axisLabel: { color: "#94a3b8", hideOverlap: true, margin: 10 },
    },
    yAxis: [
      {
        type: "value",
        name: "用量",
        minInterval: 1,
        axisLabel: { color: "#94a3b8", margin: 10 },
        splitLine: { lineStyle: { color: "#2d3748", type: "dashed" } },
      },
      {
        type: "value",
        name: "命中率",
        min: 0,
        max: 100,
        axisLabel: { formatter: "{value}%", color: "#94a3b8", margin: 10 },
        splitLine: { show: false },
      },
    ],
    series: [
      {
        name: "输入",
        type: "bar",
        barMaxWidth: 24,
        itemStyle: { borderRadius: [4, 4, 0, 0] },
        data: props.points.map((point) => point.input_tokens),
      },
      {
        name: "输出",
        type: "bar",
        barMaxWidth: 24,
        itemStyle: { borderRadius: [4, 4, 0, 0] },
        data: props.points.map((point) => point.output_tokens),
      },
      {
        name: "缓存写入",
        type: "line",
        smooth: true,
        symbol: "circle",
        symbolSize: 6,
        lineStyle: { width: 2, type: "dashed" },
        data: props.points.map((point) => point.cache_write_tokens),
      },
      {
        name: "缓存读取",
        type: "line",
        smooth: true,
        symbol: "circle",
        symbolSize: 6,
        lineStyle: { width: 2, type: "dashed" },
        data: props.points.map((point) => point.cache_read_tokens),
      },
      {
        name: "缓存命中率",
        type: "line",
        yAxisIndex: 1,
        smooth: true,
        symbol: "circle",
        symbolSize: 6,
        lineStyle: { width: 2.5 },
        data: cacheHitRates.value,
      },
    ],
  });
}

onMounted(() => {
  if (!chartElement.value) return;
  chart = echarts.init(chartElement.value);
  renderChart();
  resizeObserver = new ResizeObserver(() => chart?.resize());
  resizeObserver.observe(chartElement.value);
});

watch([categories, cacheHitRates, () => props.range], renderChart);

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  chart?.dispose();
});
</script>

<template>
  <Card full-height class="dashboard-chart-card" :hoverable="false">
    <section class="dashboard-chart-panel">
      <header>
        <h2>使用趋势</h2>
      </header>
      <div ref="chartElement" class="dashboard-chart" />
    </section>
  </Card>
</template>

<style scoped>
.dashboard-chart-card {
  height: 400px;
  min-width: 0;
  overflow: hidden;
}

.dashboard-chart-panel {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  gap: var(--spacing-md);
  overflow: hidden;
}

h2 {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 15px;
}

.dashboard-chart {
  min-height: 0;
  flex: 1;
}
</style>
