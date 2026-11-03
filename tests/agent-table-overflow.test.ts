import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

test("智能体条目表格保留横向滚动以展示窄屏列", () => {
  const source = readFileSync(
    new URL("../app/components/agents/workspace/ItemList.vue", import.meta.url),
    "utf8",
  );

  expect(source).toContain("overflow-x: auto");
  expect(source).not.toContain("overflow-x: hidden");
  expect(source).toContain(
    '{ key: "sourcePath", title: "位置", minWidth: 360, ellipsis: true }',
  );
  expect(source).toContain('{ key: "source", title: "来源", width: 96 }');
  expect(
    source.indexOf('{ key: "source", title: "来源", width: 96 }'),
  ).toBeLessThan(source.indexOf('title: "包含内容"'));
  expect(source).toContain("CONTENTS_COLUMN_WIDTH");
  expect(source).toContain("width: CONTENTS_COLUMN_WIDTH");
  expect(source).not.toContain("showStatus");
  expect(source).toContain("showVersion");
  expect(source).toContain('v-if="showVersion"');
  expect(source).toContain("function sourceLabel");
  expect(source).toContain('system: "内置"');
  expect(source).toContain("sourceLabel(row.source)");
  expect(source).toContain("sourceSemantic(row.source)");
  expect(source).toContain('class="agent-item-source__copy"');
  expect(source).toContain("flex: 0 0 auto");
  expect(source).toContain(":deep(.inline-block)");
  expect(source).toContain("max-width: 100%");
});
