import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

const cell = read("../app/components/providers/quota/QuotaCell.vue");
const rings = read("../app/components/providers/quota/PlanRings.vue");
const list = read("../app/components/providers/List.vue");

test("额度列按套餐、余额、共享和未接入四类紧凑展示", () => {
  expect(cell).toContain("QuotaPlanRings");
  expect(cell).toContain("ProgressBar");
  expect(cell).toContain("Tooltip");
  expect(cell).toContain('mode === "shared_limited"');
  expect(cell).toContain("balance");
  expect(cell).toContain("window.display");
  expect(cell).toContain('"未接入"');
  expect(cell).toContain('"待配置"');
  expect(cell).toContain('"查询失败"');
  expect(cell).toContain("stale");
});

test("套餐额度按实际窗口画同心圆环", () => {
  expect(rings).toContain("windows.slice(0, radii.length)");
  expect(rings).toContain("100 - window.used_percent");
  expect(rings).toContain('pathLength="100"');
  expect(rings).toContain("stroke-dasharray");
  expect(rings).toContain("--st-danger");
  expect(rings).toContain("--st-warning");
  expect(rings).toContain("--st-primary");
});

test("供应商列表把额度列放在创建人和操作之间并恢复共享入口", () => {
  expect(list).toContain('key: "quota"');
  expect(list).toContain('title: "额度"');
  expect(list).toContain("ProviderQuotaCell");
  expect(list).toContain("emit('share', row)");
  expect(list).not.toContain('key: "visibility"');
});
