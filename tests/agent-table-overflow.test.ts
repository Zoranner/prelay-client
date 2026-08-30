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
    '{ key: "sourcePath", title: "位置", minWidth: 360, ellipsis: true }',
  );
  expect(source).toContain('{ key: "source", title: "来源", width: 96 }');
  expect(
    source.indexOf('{ key: "status", title: "状态", width: 88 }'),
  ).toBeLessThan(source.indexOf('{ key: "source", title: "来源", width: 96 }'));
  expect(source).toContain("showStatus");
  expect(source).toContain('row.source === "team" ? "团队" : "个人"');
  expect(source).toContain('class="agent-item-source__copy"');
  expect(source).toContain("flex: 0 0 auto");
  expect(source).toContain(":deep(.inline-block)");
  expect(source).toContain("max-width: 100%");
});
