import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const page = (name: string) =>
  readFileSync(new URL(`../app/pages/${name}.vue`, import.meta.url), "utf8");

test("活动页只读取和展示活动明细", () => {
  const stats = page("stats");
  const table = readFileSync(
    new URL("../app/components/activity/RequestTable.vue", import.meta.url),
    "utf8",
  );
  const tableStyles = readFileSync(
    new URL("../app/components/activity/RequestTable.css", import.meta.url),
    "utf8",
  );
  const requestLogDto = readFileSync(
    new URL("../crates/protocol/src/stats.rs", import.meta.url),
    "utf8",
  );
  const relayStore = readFileSync(
    new URL("../app/stores/relay.ts", import.meta.url),
    "utf8",
  );
  expect(stats).toContain('"stats_activities"');
  expect(stats).not.toContain('"stats_overview"');
  expect(stats).not.toContain('"stats_models"');
  expect(stats).not.toContain('"stats_providers"');
  expect(stats).not.toContain("activeView");
  expect(stats).not.toContain("概览");
  expect(stats).toContain("RequestTable");
  expect(table).toContain("latency_ms");
  expect(table).toContain('title: "输入"');
  expect(table).toContain('title: "输出"');
  expect(table).toContain('title: "耗时"');
  expect(table).not.toContain('title: "缓存读"');
  expect(table).not.toContain('title: "缓存写"');
  expect(table).not.toContain('title: "首 Token"');
  expect(table).not.toContain('title: "总耗时"');
  expect(table).toContain("协议");
  expect(table).not.toContain("端点");
  expect(table).toContain("模式");
  expect(table).toContain("接入点");
  expect(table).toContain("供应商 / 上游模型");
  expect(table).not.toContain('<th class="text-left">错误</th>');
  expect(table).not.toContain("上游请求 ID");
  expect(table).toContain("cell-status");
  expect(table).toContain("查看活动错误");
  expect(table).not.toContain("查看活动诊断");
  expect(table).toContain("v-if=\"row.status === 'failed'\"");
  expect(table).toContain("activity-provider-model");
  expect(table).toContain("activity-metric");
  expect(table).toContain("activity-content-nowrap");
  expect(table).toContain('layout="auto"');
  expect(table).not.toMatch(
    /key: "input", title: "输入", width: \d+, ellipsis: true/,
  );
  expect(table).not.toMatch(
    /key: "output", title: "输出", width: \d+, ellipsis: true/,
  );
  expect(table).not.toMatch(
    /key: "latency", title: "耗时", width: \d+, ellipsis: true/,
  );
  expect(table).not.toContain("max-content");
  expect(table).toContain("Modal");
  expect(table).toContain("errorDialogOpen");
  expect(table).not.toContain("<details");
  expect(table).toContain("requestColumns");
  expect(table).toContain("fixed-header");
  expect(table).toContain("activity-table__grid");
  expect(tableStyles).toContain("overflow: auto");
  expect(table).not.toContain("<EmptyState");
  expect(table).toContain('empty-text="暂无活动"');
  expect(table).toContain(':loading="pending"');
  expect(table).toContain("RadioGroup");
  expect(table).toContain('variant="button"');
  expect(table).not.toMatch(
    /<RadioGroup\b[^>]*\baria-label="状态筛选"[^>]*\bsize="small"/,
  );
  expect(table).not.toMatch(
    /<Select\b[^>]*\baria-label="显示条数"[^>]*\bsize="small"/,
  );
  expect(table).toContain('class="activity-toolbar__controls"');
  expect(table).toContain('class="activity-toolbar__limit"');
  expect(table).not.toContain('v-for="option in statusOptions"');
  const columns = [
    "时间",
    "接入点",
    "供应商 / 上游模型",
    "协议",
    "模式",
    "状态",
    "输入",
    "输出",
    "耗时",
  ];
  for (const [index, column] of columns.entries()) {
    expect(table.indexOf(`title: "${column}"`)).toBeGreaterThan(
      index === 0 ? 0 : table.indexOf(`title: "${columns[index - 1]}"`),
    );
  }
  expect(table).not.toContain("request-error__message");
  expect(requestLogDto).toContain("pub struct ActivitySummary");
  expect(requestLogDto).toContain("pub endpoint_name: Option<String>");
  expect(requestLogDto).toContain("pub first_token_ms: Option<i64>");
  expect(requestLogDto).toContain("pub cache_read_tokens: Option<i64>");
  expect(requestLogDto).toContain("pub cache_write_tokens: Option<i64>");
  expect(relayStore).toContain("export interface Activity");
  expect(relayStore).not.toContain("export interface RequestLog");
  expect(relayStore).not.toContain("metadata_json");
  expect(relayStore).not.toContain("estimated_cost");
});

test("活动和仪表盘优先显示模型显示名并回退到目录或 ID", () => {
  const activity = readFileSync(
    new URL("../app/components/activity/RequestTable.vue", import.meta.url),
    "utf8",
  );
  const breakdown = readFileSync(
    new URL(
      "../app/components/dashboard/StatsBreakdownTable.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const dashboard = page("index");
  const activityPage = page("stats");
  const fixtures = {
    activity: {
      model_requested: "requested-model-id",
      model_requested_display_name: "请求模型显示名",
      model_upstream: "upstream-model-id",
      model_upstream_display_name: "上游模型显示名",
    },
    legacyActivity: {
      model_requested: "legacy-requested-model-id",
      model_upstream: "legacy-upstream-model-id",
    },
    modelStats: {
      model_requested: "stats-model-id",
      model_requested_display_name: "统计模型显示名",
    },
    legacyModelStats: {
      model_requested: "legacy-stats-model-id",
    },
  };

  expect(fixtures.activity.model_requested_display_name).toBe("请求模型显示名");
  expect(fixtures.activity.model_requested).toBe("requested-model-id");
  expect(fixtures.legacyActivity.model_requested_display_name).toBeUndefined();
  expect(fixtures.legacyActivity.model_requested).toBe(
    "legacy-requested-model-id",
  );
  expect(fixtures.modelStats.model_requested_display_name).toBe(
    "统计模型显示名",
  );
  expect(
    fixtures.legacyModelStats.model_requested_display_name,
  ).toBeUndefined();

  expect(activity).toContain("model_requested_display_name");
  expect(activity).toContain("model_upstream_display_name");
  expect(activity).toContain("modelCatalogLabel");
  expect(activity).toContain("row.model_requested_display_name");
  expect(activity).toContain("row.model_upstream_display_name");
  expect(activity).toContain("requestedModelTitle(row)");
  expect(activity).toContain("upstreamModelTitle(row)");
  expect(breakdown).toContain("model_requested_display_name");
  expect(breakdown).toContain("modelCatalogLabel");
  expect(breakdown).toContain("{{ rowName(row) }}");
  expect(dashboard).toContain("model_requested_display_name");
  expect(dashboard).toContain("modelCatalogLabel");

  expect(activityPage).toContain(
    'invokeCommand<Activity[]>("stats_activities"',
  );
  expect(activityPage).toContain("limit: limit.value");
  expect(dashboard).toContain(
    'invokeCommand<ModelStats[]>("stats_models", range)',
  );
  expect(dashboard).toContain("const range = { range: selectedRange.value };");
  expect(dashboard).toContain("row.model_requested");
  expect(dashboard).toContain(
    'id: `${row.model_requested ?? "unknown"}-${index}`',
  );
  expect(dashboard).not.toContain(
    "model_requested_display_name: selectedRange",
  );
});
