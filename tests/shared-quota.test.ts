import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

const cell = read("../app/components/providers/quota/QuotaCell.vue");
const shared = read("../app/components/providers/quota/SharedQuotaCell.vue");
const rings = read("../app/components/providers/quota/QuotaRings.vue");
const list = read("../app/components/providers/List.vue");
const format = read("../app/utils/provider/quota.ts");
const copy = read("../app/utils/provider/quota/copy.ts");
const view = read("../app/utils/provider/quota/view.ts");

test("共享限额使用独立单元格展示周期与剩余 tokens", () => {
  expect(shared).toContain("ProviderSharedQuotaCell");
  expect(shared).toContain("buildQuotaView");
  expect(shared).toContain("QuotaRings");
  expect(view).toContain("共享限额");
  expect(shared).toContain("sharedView.primary.label");
  expect(shared).toContain("sharedView.tooltip.rows");
  expect(shared).toContain("剩 {{ row.value }}");
  expect(copy).toContain('"当天"');
  expect(copy).toContain('"本周累计"');
  expect(copy).toContain('"本月累计"');
  expect(shared).toContain("ProgressBar");
  expect(shared).toContain("Tooltip");
  expect(shared).toContain("QuotaRings");
  expect(shared).not.toContain("slotFor");
  expect(shared).not.toContain("unitOrder");
  expect(shared).not.toContain("window.label");
});

test("共享圆环只分当天与累积两个槽，未配置累积保留灰槽", () => {
  expect(rings).toContain("slots.length >= 3");
  expect(rings).toContain("[6, 9.5]");
  expect(rings).toContain("slot.configured");
  expect(rings).toContain("quota-rings__track--empty");
  expect(rings).toContain("stroke-dasharray");
  expect(rings).toContain('pathLength="100"');
  expect(rings).toContain("slot.percent");
});

test("共享限额不再进入套餐额度单元格", () => {
  expect(cell).not.toContain("shared_limited");
  expect(list).toContain("row.quota?.mode === 'shared_limited'");
  expect(list).toContain("ProviderSharedQuotaCell");
  expect(list).toContain("ProviderQuotaCell");
});

test("额度百分比与倒计时格式在两种单元格间共用", () => {
  expect(format).toContain('export * from "./quota/model"');
  expect(format).toContain('export * from "./quota/format"');
  expect(cell).toContain("~/utils/provider/quota");
  expect(shared).toContain("~/utils/provider/quota");
});
