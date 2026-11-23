import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

const listSource = read("../app/components/providers/List.vue");
const formSource = read("../app/components/providers/Form.vue");
const formLogic = read("../app/composables/providerForm.ts");
const pageSource = read("../app/pages/providers.vue");
const templatesSource = read("../app/utils/provider/templates.ts");

test("供应商列表展示弃用状态并禁用编辑分享", () => {
  expect(listSource).toContain("row.deprecated");
  expect(listSource).toContain(">已弃用</Tag");
  const disabled = ':disabled="!row.isOwn || row.deprecated"';
  expect(listSource.split(disabled).length - 1).toBe(2);
});

test("新建供应商不再提供弃用类型", () => {
  expect(formLogic).toContain("!option.deprecated");
  expect(templatesSource).toContain("deprecated: provider.deprecated");
});

test("弃用供应商只显示弃用提示", () => {
  expect(formSource).toContain("该供应商已弃用，现有配置可在短期内继续使用");
  expect(formSource).not.toContain("已弃用或下架");
  expect(formLogic).toContain("该供应商的目录条目已弃用，只能删除。");
  expect(pageSource).toContain("editingProvider.value?.deprecated");
});
