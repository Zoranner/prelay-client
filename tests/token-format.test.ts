import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import { formatTokens, formatTrendValue } from "../app/utils/stats/tokenFormat";

test("Token 数量按 K、M、B 显示", () => {
  expect(formatTokens(999)).toBe("999");
  expect(formatTokens(1_500)).toBe("1.5K");
  expect(formatTokens(1_500_000)).toBe("1.5M");
  expect(formatTokens(3_671_800_000)).toBe("3.7B");
});

test("使用趋势纵轴复用 Token 数量格式", () => {
  const chart = readFileSync(
    new URL(
      "../app/components/dashboard/charts/TokenTrend.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(chart).toContain('from "~/utils/stats/tokenFormat"');
  expect(chart).toContain("formatter: formatTokens");
});

test("使用趋势 tooltip 显示 Token 单位并保留命中率百分号", () => {
  expect(formatTrendValue(602_579_044)).toBe("602.6M");
  expect(formatTrendValue(776_617)).toBe("776.6K");
  expect(formatTrendValue(99.8, true)).toBe("99.8%");
  expect(formatTrendValue(100, true)).toBe("100%");

  const chart = readFileSync(
    new URL(
      "../app/components/dashboard/charts/TokenTrend.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(chart).toContain("formatter: tooltipFormatter");
  expect(chart).toContain("formatTrendValue(");
  expect(chart).toContain("axisValueLabel");
});

test("供应商和模型统计表复用 Token 数量格式", () => {
  const table = readFileSync(
    new URL(
      "../app/components/dashboard/stats/BreakdownTable.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(table).toContain(
    'import { formatTokens } from "~/utils/stats/tokenFormat"',
  );
  expect(table).not.toContain("function formatTokens(value: number)");
});
