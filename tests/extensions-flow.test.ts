import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("扩展表格显示仓库、版本和可复制的参考来源", () => {
  const catalogTable = source("components/extensions/ExtensionCatalogTable.vue");

  expect(catalogTable).toContain("function repositoryUrl");
  expect(catalogTable).toContain("function copyRepositoryUrl");
  expect(catalogTable).toContain('key: "repository", title: "仓库"');
  expect(catalogTable).toContain('key: "source", title: "参考来源"');
  expect(catalogTable).toContain("#cell-repository");
  expect(catalogTable).toContain("#cell-source");
  expect(catalogTable).toContain("repositoryUrl(row.repository)");
  expect(catalogTable).toContain('target="_blank"');
  expect(catalogTable).toContain('icon="ph:copy"');
  expect(catalogTable).toContain("copyRepositoryUrl(row.repository)");
  expect(catalogTable).not.toContain('icon="ph:arrow-square-out"');
  expect(catalogTable).not.toContain('key: "summary"');
  expect(catalogTable).not.toContain("row.summary");
  expect(catalogTable).not.toContain("row.name");
});

test("扩展库沿用智能体工作区的分类表格与单层操作表面", () => {
  const page = source("pages/agents.vue");
  const detailDrawer = source("components/extensions/ExtensionDetailDrawer.vue");
  const installModal = source("components/extensions/ExtensionInstallModal.vue");

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
  expect(page).toMatch(
    /function selectExtensionCatalog\(\) \{\s+activeWorkspace\.value = "extensions";\s+void extensionCatalog\.load\(true\);/,
  );
  expect(page).not.toContain("showExtensionCatalog");
  expect(page).not.toContain("<small>agents</small>");
  expect(detailDrawer).toContain("MarkdownViewer");
  expect(detailDrawer).toContain('title="扩展详情"');
  expect(detailDrawer).toContain('<template #footer>');
  expect(detailDrawer).toContain(">关闭</Button>");
  expect(detailDrawer).not.toContain("extension-detail__meta");
  expect(detailDrawer).not.toContain("risk");
  expect(detailDrawer).not.toContain(':show-footer="false"');
  expect(detailDrawer).toContain(".extension-detail__readme :deep(h2)");
  expect(detailDrawer).toContain(".extension-detail__readme :deep(p)");
  expect(detailDrawer).not.toContain("white-space: pre-wrap");
  expect(installModal).toContain("<Select");
  expect(installModal).toContain('label="安装到"');
  expect(installModal).toContain("multiple");
  expect(installModal).toContain("function selectClients");
  expect(installModal).not.toContain("Checkbox");
  expect(installModal).not.toContain("extension_install_preview");
  expect(installModal).not.toContain("正在生成安装变更");
});
