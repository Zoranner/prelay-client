import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

const cell = read("../app/components/providers/quota/QuotaCell.vue");
const rings = read("../app/components/providers/quota/QuotaRings.vue");
const list = read("../app/components/providers/List.vue");

test("额度列按套餐、余额和未接入紧凑展示", () => {
  expect(cell).toContain("QuotaRings");
  expect(cell).toContain("buildQuotaView");
  expect(cell).toContain("Tooltip");
  expect(cell).toContain("ProgressBar");
  expect(cell).toContain("Spinner");
  expect(cell).toContain("quota-cell__skeleton");
  expect(cell).toContain("ph:wallet");
  expect(cell).toContain("quota-cell__lead");
  expect(cell).toContain("quota-cell__window");
  expect(cell).toContain("quota-cell__balance");
  expect(cell).toContain("quota-cell__status--chip");
  expect(cell).toContain("quota-tooltip__status");
  expect(cell).toContain("contentView.tooltip.rows");
  expect(cell).toContain("statusTone");
  expect(cell).toContain("stale");
  expect(cell).not.toContain("unitOrder");
  expect(cell).not.toContain("window.label");
  expect(cell).not.toContain("balance.text");
  expect(cell).not.toContain("formatQuotaAmount");
});

test("套餐额度按实际窗口画同心圆环", () => {
  expect(rings).toContain("radii");
  expect(rings).toContain("slots");
  expect(rings).toContain("configured");
  expect(rings).toContain("quota-rings__track--empty");
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
