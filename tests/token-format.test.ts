import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import { formatTokens } from "../app/utils/tokenFormat";

test("Token 数量按 K、M、B 显示", () => {
  expect(formatTokens(999)).toBe("999");
  expect(formatTokens(1_500)).toBe("1.5K");
  expect(formatTokens(1_500_000)).toBe("1.5M");
  expect(formatTokens(3_671_800_000)).toBe("3.7B");
});

test("使用趋势纵轴复用 Token 数量格式", () => {
  const chart = readFileSync(
    new URL(
      "../app/components/dashboard/TokenUsageTrendChart.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(chart).toContain('import { formatTokens } from "~/utils/tokenFormat"');
  expect(chart).toContain("formatter: formatTokens");
});

test("供应商和模型统计表复用 Token 数量格式", () => {
  const table = readFileSync(
    new URL(
      "../app/components/dashboard/StatsBreakdownTable.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(table).toContain('import { formatTokens } from "~/utils/tokenFormat"');
  expect(table).not.toContain("function formatTokens(value: number)");
});
