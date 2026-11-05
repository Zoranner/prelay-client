import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

test("插件列表按安装包呈现版本、内容和位置", () => {
  const source = readFileSync(
    new URL(
      "../app/components/agents/workspace/items/PluginList.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const row = readFileSync(
    new URL(
      "../app/components/agents/workspace/items/AgentItemRow.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(source).not.toContain("<Table");
  expect(source).toContain("AgentItemRow");
  expect(source).toContain('class="agent-item-list"');
  expect(source).toContain('icon="ph:puzzle-piece"');
  expect(source).toContain("agent-meta-badge");
  expect(source).toContain("item.artifacts");
  expect(source).toContain("agent-artifact-lines");
  expect(source).toContain("<TagGroup");
  expect(source).not.toContain("agent-artifact-tags");
  expect(source).not.toContain("ItemPath");
  expect(source).toContain("<EmptyState");
  expect(source).toContain('title="暂无插件"');
  expect(row).toContain("width: 45px");
  expect(row).toContain("height: 45px");
  expect(row).toContain("grid-template-columns: 45px minmax(0, 1fr)");
  expect(row).toContain("padding-left: calc(45px + var(--spacing-md))");
  expect(row).toContain("agent-item-row__path");
  expect(row).toContain("agent-item-row__icon--plugin");
  expect(row).not.toContain("agent-item-row__icon--mcp");
  expect(row).not.toContain("agent-item-row__icon--skill");
  expect(row).not.toContain("agent-item-row__icon--hook");
});

test("插件内的 Skill、MCP、Hook 分组不换行", () => {
  const plugin = readFileSync(
    new URL(
      "../app/components/agents/workspace/items/PluginList.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const styles = readFileSync(
    new URL(
      "../app/components/agents/workspace/items/items.css",
      import.meta.url,
    ),
    "utf8",
  );
  const mcp = readFileSync(
    new URL(
      "../app/components/agents/workspace/items/McpList.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const hook = readFileSync(
    new URL(
      "../app/components/agents/workspace/items/HookList.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const skill = readFileSync(
    new URL(
      "../app/components/agents/workspace/items/SkillList.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(plugin).toContain("<TagGroup");
  expect(plugin).toContain(':items="group.items"');
  expect(styles).not.toContain("overflow-x: auto");
  expect(mcp).toContain(':title="item.target"');
  expect(mcp).toContain("agent-detail-line--ellipsis");
  expect(mcp).toContain("-");
  expect(hook).toContain("agent-detail-line--ellipsis");
  expect(skill).toContain("v-else");
  expect(skill).toContain(">");
  expect(skill).toContain("-");
});

test("页面列表只投影真实扫描结果并移除不可读取条目", () => {
  const selection = readFileSync(
    new URL("../app/composables/agent/pageSelection.ts", import.meta.url),
    "utf8",
  );
  const workspace = readFileSync(
    new URL("../app/composables/agent/workspace.ts", import.meta.url),
    "utf8",
  );
  const app = readFileSync(new URL("../app/app.vue", import.meta.url), "utf8");

  expect(selection).toContain("hasUnresolvedError");
  expect(selection).toContain("!hasUnresolvedError(item)");
  expect(workspace).not.toContain("mockItems");
  expect(workspace).not.toContain("isAgentMockPreview");
  expect(app).not.toContain("isAgentMockPreview");
});
