import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

const listSource = read("../app/components/providers/List.vue");
const formSource = read("../app/components/providers/Form.vue");
const formLogic = read("../app/composables/providerForm.ts");
const pageSource = read("../app/pages/providers.vue");
const templatesSource = read("../app/utils/provider/templates.ts");

test("供应商列表用删除线展示弃用状态并允许打开只读配置", () => {
  expect(listSource).toContain("row.deprecated");
  expect(listSource).toContain("provider-name-text--retired");
  expect(listSource).not.toContain(">已弃用</Tag");
  expect(listSource).toMatch(
    /:disabled="!row\.isOwn"\s+aria-label="编辑供应商"/,
  );
  expect(listSource).toMatch(
    /:disabled="!row\.isOwn \|\| row\.deprecated"\s+aria-label="共享供应商"/,
  );
  expect(pageSource).toContain("!editingRetired");
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
