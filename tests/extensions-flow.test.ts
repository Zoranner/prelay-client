import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("扩展库沿用智能体工作区的分类表格与单层操作表面", () => {
  const page = source("pages/agents.vue");

  expect(page).toContain("扩展库");
  expect(page).toContain('<ExtensionCatalogTable');
  expect(page).toContain('<ExtensionDetailDrawer');
  expect(page).toContain('<ExtensionInstallModal');
  expect(page).toContain('value: "rule"');
  expect(page).toContain('value: "plugin"');
  expect(page).toContain('value: "mcp"');
  expect(page).toContain('value: "skill"');
  expect(page).toContain('type AgentWorkspace = AgentClient | "extensions";');
  expect(page).toContain(':active="activeWorkspace === client.client"');
  expect(page).toContain(':active="activeWorkspace === \'extensions\'"');
  expect(page).not.toContain("showExtensionCatalog");
  expect(page).not.toContain("<small>agents</small>");
});
