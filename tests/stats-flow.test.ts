import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const page = (name: string) =>
  readFileSync(new URL(`../app/pages/${name}.vue`, import.meta.url), "utf8");

test("活动页只读取和展示请求明细", () => {
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
  expect(stats).toContain('"stats_requests"');
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
  expect(table).toContain("查看请求诊断");
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
  expect(table).toContain("diagnosticsDialogOpen");
  expect(table).not.toContain("<details");
  expect(table).toContain("requestColumns");
  expect(table).toContain("fixed-header");
  expect(table).toContain("activity-table__grid");
  expect(tableStyles).toContain("overflow: auto");
  expect(table).not.toContain("<EmptyState");
  expect(table).toContain('empty-text="暂无请求记录"');
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
  expect(requestLogDto).toContain("pub endpoint_name: Option<String>");
  expect(requestLogDto).toContain("pub first_token_ms: Option<i64>");
  expect(requestLogDto).toContain("pub cache_read_tokens: Option<i64>");
  expect(requestLogDto).toContain("pub cache_write_tokens: Option<i64>");
});
