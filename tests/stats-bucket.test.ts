import { expect, test } from "bun:test";

import { parseTimelineBucket } from "../app/utils/stats";

test("时间桶兼容 SQLite 本地时间和 ISO 时间字符串", () => {
  expect(parseTimelineBucket("2026-08-22 08:00:00")?.toISOString()).toBe(
    "2026-08-22T00:00:00.000Z",
  );
  expect(parseTimelineBucket("2026-08-22T00:00:00Z")?.toISOString()).toBe(
    "2026-08-22T00:00:00.000Z",
  );
  expect(parseTimelineBucket("2026-08-22")?.toISOString()).toBe(
    "2026-08-21T16:00:00.000Z",
  );
});

test("无效时间桶不会传给图表格式化器", () => {
  expect(parseTimelineBucket("not-a-time")).toBeNull();
});
