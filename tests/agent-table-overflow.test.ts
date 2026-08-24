import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

test("智能体条目表格保留横向滚动以展示窄屏列", () => {
  const source = readFileSync(
    new URL("../app/components/agents/AgentItemList.vue", import.meta.url),
    "utf8",
  );

  expect(source).toContain("overflow-x: auto");
  expect(source).not.toContain("overflow-x: hidden");
  expect(source).toContain(
    '{ key: "sourcePath", title: "来源", minWidth: 360, ellipsis: true }',
  );
  expect(source).toContain('class="agent-item-source__copy"');
  expect(source).toContain("flex: 0 0 auto");
  expect(source).toContain(":deep(.inline-block)");
  expect(source).toContain("max-width: 100%");
});
