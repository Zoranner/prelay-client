import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("桌面壳层按工作流组织客户端入口", () => {
  const app = source("app.vue");
  const navigation = source("components/workbench/WorkbenchShell.vue");

  expect(app).toContain("WorkbenchShell");
  expect(navigation).toContain("SidebarItem");
  expect(navigation).toContain('label: "工作台"');
  expect(navigation).toContain('label: "供应商"');
  expect(navigation).toContain('label: "接入点"');
  expect(navigation).toContain('label: "活动"');
  expect(navigation).toContain("设置");
  expect(app).not.toContain("app-nav");
});

test("工作台按选定范围展示指标、趋势和统计列表", () => {
  const page = source("pages/index.vue");
  const overview = source("components/dashboard/StatsOverview.vue");
  const rangeSelect = source("components/dashboard/StatsRangeSelect.vue");
  const table = source("components/dashboard/StatsBreakdownTable.vue");
  const tokenUsageTrend = source("components/dashboard/TokenUsageTrendChart.vue");

  expect(page).toContain('"stats_overview"');
  expect(page).toContain('"stats_models"');
  expect(page).toContain('"stats_providers"');
  expect(page).toContain('"stats_timeline"');
  expect(page).toContain("StatsRangeSelect");
  expect(page).toContain('const selectedRange = ref<StatsRange>("today")');
  expect(page).toContain("{ range: selectedRange.value }");
  expect(page).not.toContain("ModelDistributionChart");
  expect(page).toContain("TokenUsageTrendChart");
  expect(page).toContain("StatsBreakdownTable");
  expect(overview).toContain("平均响应");
  expect(overview).toContain("缓存命中率");
  expect(overview).toContain("请求数");
  expect(overview).toContain("总 Token");
  expect(overview).toContain("输入 Token");
  expect(overview).toContain("输出 Token");
  expect(overview).toContain("grid-template-columns: repeat(6, minmax(0, 1fr))");
  expect(overview).not.toContain("今日请求");
  expect(overview).not.toContain("累计 Token");
  expect(rangeSelect).toContain("今日");
  expect(rangeSelect).toContain("昨日");
  expect(rangeSelect).toContain("本周");
  expect(rangeSelect).toContain("上周");
  expect(rangeSelect).toContain("本月");
  expect(rangeSelect).toContain("上月");
  expect(rangeSelect).toContain("本年");
  expect(rangeSelect).toContain("去年");
  expect(rangeSelect).toContain("总计");
  expect(page).toContain("供应商统计");
  expect(page).toContain("模型统计");
  expect(table).toContain("fixed-header");
  expect(table).toContain("height: 272px");
  expect(tokenUsageTrend).toContain('from "echarts"');
  expect(tokenUsageTrend).toContain("full-height");
  expect(tokenUsageTrend).toContain("height: 400px");
  expect(tokenUsageTrend).toContain('left: "center"');
  expect(tokenUsageTrend).not.toContain('icon: "roundRect"');
  expect(tokenUsageTrend).toContain("fontSize: 14");
  expect(tokenUsageTrend).toContain("borderRadius: [4, 4, 0, 0]");
  expect(tokenUsageTrend).not.toContain('stack: "Token 用量"');
  expect(tokenUsageTrend).toContain("alignWithLabel: true");
  expect(tokenUsageTrend).toContain("使用趋势");
  expect(tokenUsageTrend).not.toContain("最近 7 天输入、输出与缓存 Token");
  expect(tokenUsageTrend).toContain('name: "输入"');
  expect(tokenUsageTrend).toContain('name: "输出"');
  expect(tokenUsageTrend).toContain('name: "缓存写入"');
  expect(tokenUsageTrend).toContain('name: "缓存读取"');
  expect(tokenUsageTrend).toContain("缓存命中率");
  expect(tokenUsageTrend).not.toContain('name: "输入 Token"');
  expect(tokenUsageTrend).not.toContain('name: "输出 Token"');
  expect(tokenUsageTrend).not.toContain('name: "缓存写入 Token"');
  expect(tokenUsageTrend).not.toContain('name: "缓存读取 Token"');
  expect(tokenUsageTrend).toContain("type: \"line\"");
});

test("首次启动和服务配置拥有独立页面", () => {
  expect(existsSync(new URL("../app/pages/setup.vue", import.meta.url))).toBe(
    true,
  );
  expect(
    existsSync(new URL("../app/pages/settings.vue", import.meta.url)),
  ).toBe(true);
});
