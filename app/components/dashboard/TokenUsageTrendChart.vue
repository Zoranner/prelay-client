<script setup lang="ts">
import { Card } from "@stellar/ui";
import type * as ECharts from "echarts";

import type { StatsRange, TokenUsageTimelinePoint } from "~/stores/relay";
import { parseTimelineBucket } from "~/utils/stats";

const props = defineProps<{
  points: TokenUsageTimelinePoint[];
  range: StatsRange;
}>();

const chartElement = ref<HTMLElement | null>(null);
let chart: ECharts.ECharts | undefined;
let resizeObserver: ResizeObserver | undefined;
let echarts: typeof import("echarts") | undefined;
let disposed = false;

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

const categories = computed(() =>
  props.points.map((point) => formatBucket(point.bucket)),
);
const cacheHitRates = computed(() =>
  props.points.map((point) => {
    const totalInput = point.total_input_tokens;
    return totalInput
      ? Number(((point.cache_read_tokens / totalInput) * 100).toFixed(1))
      : 0;
  }),
);

function themeColor(name: string) {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
}

function renderChart() {
  if (!chart) return;
  const colors = {
    primary: themeColor("--st-primary"),
    success: themeColor("--st-success"),
    warning: themeColor("--st-warning"),
    info: themeColor("--st-info"),
    danger: themeColor("--st-danger"),
    elevated: themeColor("--st-bg-elevated"),
    border: themeColor("--st-border"),
    borderSubtle: themeColor("--st-border-subtle"),
    divider: themeColor("--st-border-divider"),
    primaryText: themeColor("--st-text-primary"),
    secondaryText: themeColor("--st-text-secondary"),
    mutedText: themeColor("--st-text-muted"),
  };
  chart.setOption({
    color: [
      colors.primary,
      colors.success,
      colors.warning,
      colors.info,
      colors.danger,
    ],
    grid: { top: 62, right: 54, bottom: 40, left: 54 },
    legend: {
      top: 0,
      left: "center",
      itemWidth: 22,
      itemHeight: 12,
      itemGap: 20,
      textStyle: { color: colors.secondaryText, fontSize: 14 },
    },
    tooltip: {
      trigger: "axis",
      backgroundColor: colors.elevated,
      borderColor: colors.border,
      borderWidth: 1,
      padding: [10, 12],
      textStyle: { color: colors.primaryText },
    },
    xAxis: {
      type: "category",
      data: categories.value,
      boundaryGap: true,
      axisLine: { lineStyle: { color: colors.borderSubtle } },
      axisTick: { show: false, alignWithLabel: true },
      axisLabel: { color: colors.mutedText, hideOverlap: true, margin: 10 },
    },
    yAxis: [
      {
        type: "value",
        name: "用量",
        minInterval: 1,
        axisLabel: { color: colors.mutedText, margin: 10 },
        splitLine: { lineStyle: { color: colors.divider, type: "dashed" } },
      },
      {
        type: "value",
        name: "命中率",
        min: 0,
        max: 100,
        axisLabel: {
          formatter: "{value}%",
          color: colors.mutedText,
          margin: 10,
        },
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

onMounted(async () => {
  if (!chartElement.value) return;
  echarts = await import("echarts");
  if (disposed || !chartElement.value || !echarts) return;
  chart = echarts.init(chartElement.value);
  renderChart();
  window.addEventListener("prelay:theme-changed", renderChart);
  resizeObserver = new ResizeObserver(() => chart?.resize());
  resizeObserver.observe(chartElement.value);
});

watch([categories, cacheHitRates, () => props.range], renderChart);

onBeforeUnmount(() => {
  disposed = true;
  window.removeEventListener("prelay:theme-changed", renderChart);
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
