import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

const cell = read("../app/components/providers/quota/QuotaCell.vue");
const rings = read("../app/components/providers/quota/PlanRings.vue");
const list = read("../app/components/providers/List.vue");

test("额度列按套餐、余额、共享和未接入四类紧凑展示", () => {
  expect(cell).toContain("QuotaPlanRings");
  expect(cell).toContain("Tooltip");
  expect(cell).toContain("ProgressBar");
  expect(cell).toContain("Spinner");
  expect(cell).toContain("quota-cell__skeleton");
  expect(cell).toContain("ph:wallet");
  expect(cell).toContain("ph:plugs");
  expect(cell).toContain("shared_limited");
  expect(cell).toContain("quota-cell__lead");
  expect(cell).toContain("quota-cell__window");
  expect(cell).toContain("quota-cell__balance");
  expect(cell).toContain("quota-cell__status--chip");
  expect(cell).toContain("quota-tooltip__status");
  expect(cell).toContain("unitOrder");
  expect(cell).toContain("tightestWindow");
  expect(cell).toContain("balance");
  expect(cell).toContain("window.display");
  expect(cell).toContain('"未接入"');
  expect(cell).toContain('"待配置"');
  expect(cell).toContain('"查询失败"');
  expect(cell).toContain("stale");
});

test("套餐额度按实际窗口画同心圆环", () => {
  expect(rings).toContain("radii.map");
  expect(rings).toContain("slotFor");
  expect(rings).toContain("disabled");
  expect(rings).toContain("quota-rings__track--disabled");
  expect(rings).toContain("100 - window.used_percent");
  expect(rings).toContain('pathLength="100"');
  expect(rings).toContain("stroke-dasharray");
  expect(rings).toContain("--st-danger");
  expect(rings).toContain("--st-warning");
  expect(rings).toContain("--st-primary");
});

test("供应商列表保留协议列并把额度放在创建人之后", () => {
  expect(list).toContain('key: "quota"');
  expect(list).toContain('title: "剩余额度"');
  expect(list).toContain("width: 180");
  expect(list).toContain("ProviderQuotaCell");
  expect(list).toContain(':provider-name="row.name || row.provider_type"');
  expect(list).toContain('key: "protocols"');
  expect(list).toContain("#cell-protocols");
  expect(list.indexOf('key: "name"')).toBeLessThan(
    list.indexOf('key: "protocols"'),
  );
  expect(list.indexOf('key: "protocols"')).toBeLessThan(
    list.indexOf('key: "status"'),
  );
  expect(list.indexOf('key: "status"')).toBeLessThan(
    list.indexOf('key: "quota"'),
  );
  expect(list.indexOf('key: "quota"')).toBeLessThan(
    list.indexOf('key: "owner"'),
  );
  expect(list).toContain("emit('share', row)");
  expect(list).not.toContain('key: "visibility"');
});
