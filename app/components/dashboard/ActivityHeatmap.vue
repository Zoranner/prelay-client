<script setup lang="ts">
import { Card } from "@stellar/ui";

import type { StatsRange, TokenUsageTimelinePoint } from "~/stores/relay";
import { formatTokens } from "~/utils/tokenFormat";

const props = defineProps<{
  points: TokenUsageTimelinePoint[];
  range: StatsRange;
  summaryTokens: number;
}>();

// 本年日历形态：7 行（周一~周日）× 53 列（周），年末所在周固定在最右
const WEEKS = 53;
const weekdayLabels = ["一", "", "三", "", "五", "", ""];

const dayTotals = computed(() => {
  const totals = new Map<string, number>();
  for (const point of props.points) {
    const day = point.bucket.slice(0, 10);
    totals.set(
      day,
      (totals.get(day) ?? 0) + point.input_tokens + point.output_tokens,
    );
  }
  return totals;
});

// 桶粒度只看数据本身：小时/6 小时能归并成天；半月的桶代表一段区间，画不进"一格一天"的网格
const span = computed(() => {
  const labels = props.points.map((point) => point.bucket);
  if (!labels.length) return "day" as const;
  if (labels.some((label) => label.includes(":"))) return "day" as const;
  const halfMonthLabels =
    labels.length > 1 &&
    labels.every((label) => [1, 16].includes(Number(label.slice(8, 10))));
  return halfMonthLabels ? ("coarse" as const) : ("day" as const);
});

const peak = computed(() =>
  [...dayTotals.value.values()].reduce(
    (highest, tokens) => Math.max(highest, tokens),
    0,
  ),
);

// 高亮窗口由筛选范围决定（周=那一周、月=那个月、年=整年）；窗口外的数据照画，只是灰置
const highlight = computed(() => rangeWindow(props.range));

const columns = computed(() => {
  // 网格右端固定为本年最后一天所在的那一周；尚未到达的日期保留为 0
  const anchor = yearEnd(todayDay());
  const start = shiftDay(weekStart(anchor), -7 * (WEEKS - 1));
  return Array.from({ length: WEEKS }, (_, index) =>
    shiftDay(start, index * 7),
  );
});

// 行优先排列：一周一列、一天一行，格子宽度按可用空间等分，高度用正方比撑开
const gridRows = computed(() =>
  weekdayLabels.map((label, offset) => ({
    label,
    cells: columns.value.map((week) => {
      const day = shiftDay(week, offset);
      const tokens = dayTotals.value.get(day) ?? 0;
      return {
        day,
        tokens,
        label: dayLabel(day),
        level: level(tokens),
        dimmed: isDimmed(day),
      };
    }),
  })),
);

const monthLabels = computed(() =>
  columns.value.map((week) => {
    const month = monthStartInCurrentYear(week);
    return month ? monthLabel(month) : "";
  }),
);

// 网格起点：以数据里最晚的一天为准（服务端给的是北京时间日期），退化时用本机今天
function isDimmed(day: string) {
  const window = highlight.value;
  if (!window) return false;
  return day < window.start || day > window.end;
}

function rangeWindow(range: StatsRange) {
  const today = todayDay();
  switch (range) {
    case "today":
      return { start: today, end: today };
    case "yesterday": {
      const yesterday = shiftDay(today, -1);
      return { start: yesterday, end: yesterday };
    }
    case "this_week": {
      const start = weekStart(today);
      return { start, end: shiftDay(start, 6) };
    }
    case "last_week": {
      const start = shiftDay(weekStart(today), -7);
      return { start, end: shiftDay(start, 6) };
    }
    case "this_month": {
      const month = today.slice(0, 7);
      return { start: `${month}-01`, end: monthEnd(month) };
    }
    case "last_month": {
      const month = shiftMonth(today.slice(0, 7), -1);
      return { start: `${month}-01`, end: monthEnd(month) };
    }
    case "this_year": {
      const year = today.slice(0, 4);
      return { start: `${year}-01-01`, end: `${year}-12-31` };
    }
    case "last_year": {
      const year = Number(today.slice(0, 4)) - 1;
      return { start: `${year}-01-01`, end: `${year}-12-31` };
    }
    default:
      return null;
  }
}

function monthEnd(month: string) {
  const [year = 0, index = 1] = month.split("-").map(Number);
  return new Date(Date.UTC(year, index, 0)).toISOString().slice(0, 10);
}

function shiftMonth(month: string, offset: number) {
  const [year = 0, index = 1] = month.split("-").map(Number);
  return new Date(Date.UTC(year, index - 1 + offset, 1))
    .toISOString()
    .slice(0, 7);
}

function level(tokens: number) {
  if (!peak.value || tokens <= 0) return 0;
  return Math.max(1, Math.min(4, Math.ceil((tokens / peak.value) * 4)));
}

function todayDay() {
  const now = new Date();
  const month = `${now.getMonth() + 1}`.padStart(2, "0");
  const date = `${now.getDate()}`.padStart(2, "0");
  return `${now.getFullYear()}-${month}-${date}`;
}

function yearEnd(day: string) {
  return `${day.slice(0, 4)}-12-31`;
}

function monthStartInCurrentYear(week: string) {
  const year = todayDay().slice(0, 4);
  for (let offset = 0; offset < 7; offset += 1) {
    const day = shiftDay(week, offset);
    if (day.startsWith(year) && day.endsWith("-01")) return day.slice(0, 7);
  }
  return "";
}

function weekStart(day: string) {
  return shiftDay(day, -((weekday(day) + 6) % 7));
}

function weekday(day: string) {
  const [year = 0, month = 1, date = 1] = day.split("-").map(Number);
  return new Date(Date.UTC(year, month - 1, date)).getUTCDay();
}

function shiftDay(day: string, offset: number) {
  const [year = 0, month = 1, date = 1] = day.split("-").map(Number);
  const at = new Date(Date.UTC(year, month - 1, date));
  at.setUTCDate(at.getUTCDate() + offset);
  return at.toISOString().slice(0, 10);
}

function dayLabel(day: string) {
  const [, month, date] = day.split("-");
  return `${Number(month)}/${Number(date)}`;
}

function monthLabel(month: string) {
  return `${Number(month.slice(5, 7))}月`;
}

function cellTitle(tokens: number, label: string) {
  return `${label} · ${formatTokens(tokens)} Token`;
}
</script>

<template>
  <Card :hoverable="false">
    <section class="activity-heatmap">
      <header class="activity-heatmap__header">
        <h2>活跃度</h2>
        <span>{{ formatTokens(props.summaryTokens) }} Token</span>
      </header>
      <template v-if="span === 'coarse'">
        <p class="activity-heatmap__notice">
          当前范围内服务端只提供半月粒度的数据，按天粒度上线后这里会自动显示每天一个格子。
        </p>
      </template>
      <template v-else>
        <div class="activity-heatmap__grid">
          <span />
          <span
            v-for="(label, index) in monthLabels"
            :key="`month-${columns[index]}`"
            class="activity-heatmap__month"
          >
            {{ label }}
          </span>
          <template v-for="row in gridRows" :key="`row-${row.label}`">
            <span class="activity-heatmap__weekday">{{ row.label }}</span>
            <span
              v-for="cell in row.cells"
              :key="cell.day"
              class="activity-heatmap__cell"
              :class="[
                `activity-heatmap__cell--${cell.level}`,
                { 'activity-heatmap__cell--dimmed': cell.dimmed },
              ]"
              :title="cellTitle(cell.tokens, cell.label)"
            />
          </template>
        </div>
        <div class="activity-heatmap__legend">
          <span>少</span>
          <span
            class="activity-heatmap__cell activity-heatmap__legend-cell activity-heatmap__cell--1"
          />
          <span
            class="activity-heatmap__cell activity-heatmap__legend-cell activity-heatmap__cell--2"
          />
          <span
            class="activity-heatmap__cell activity-heatmap__legend-cell activity-heatmap__cell--3"
          />
          <span
            class="activity-heatmap__cell activity-heatmap__legend-cell activity-heatmap__cell--4"
          />
          <span>多</span>
        </div>
      </template>
    </section>
  </Card>
</template>

<style scoped>
.activity-heatmap {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: var(--spacing-md);
}

.activity-heatmap__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.activity-heatmap__header h2 {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 15px;
}

.activity-heatmap__header span {
  color: var(--st-text-secondary);
  font-size: 12px;
  white-space: nowrap;
}

.activity-heatmap__notice {
  margin: 0;
  color: var(--st-text-muted);
  font-size: 12px;
}

.activity-heatmap__month {
  overflow: visible;
  color: var(--st-text-muted);
  font-size: 11px;
  line-height: 1;
  white-space: nowrap;
}

/* 54 列：第 1 列放星期标签，其余 53 列按可用宽度等分，格子用正方比撑高 */
.activity-heatmap__grid {
  display: grid;
  min-width: 0;
  grid-template-columns: 18px repeat(53, minmax(0, 1fr));
  gap: 3px;
  align-items: center;
}

.activity-heatmap__weekday {
  color: var(--st-text-muted);
  font-size: 10px;
  line-height: 1;
  white-space: nowrap;
}

.activity-heatmap__cell {
  display: block;
  width: 100%;
  aspect-ratio: 1;
  border-radius: 3px;
  background: var(--st-bg-surface);
}

.activity-heatmap__cell--1 {
  background: color-mix(in srgb, var(--st-primary) 22%, var(--st-bg-surface));
}

.activity-heatmap__cell--2 {
  background: color-mix(in srgb, var(--st-primary) 45%, var(--st-bg-surface));
}

.activity-heatmap__cell--3 {
  background: color-mix(in srgb, var(--st-primary) 70%, var(--st-bg-surface));
}

.activity-heatmap__cell--4 {
  background: var(--st-primary);
}

.activity-heatmap__cell--dimmed {
  opacity: 0.28;
}

.activity-heatmap__legend {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--spacing-xs);
  color: var(--st-text-muted);
  font-size: 11px;
}

.activity-heatmap__legend-cell {
  width: 12px;
  height: 12px;
  aspect-ratio: auto;
  flex: 0 0 auto;
}
</style>
