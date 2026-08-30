import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("扩展表格显示名称、版本和可复制的仓库地址", () => {
  const catalogTable = source(
    "components/extensions/ExtensionCatalogTable.vue",
  );

  expect(catalogTable).toContain("function repositoryUrl");
  expect(catalogTable).toContain("function copyRepositoryUrl");
  expect(catalogTable).toContain('key: "name", title: "名称"');
  expect(catalogTable).toContain('key: "source", title: "仓库地址"');
  expect(catalogTable).toContain("#cell-name");
  expect(catalogTable).toContain("#cell-source");
  expect(catalogTable).toContain("row.name");
  expect(catalogTable).toContain("repositoryUrl(row.repository)");
  expect(catalogTable).toContain('target="_blank"');
  expect(catalogTable).toContain('icon="ph:copy"');
  expect(catalogTable).toContain("copyRepositoryUrl(row.repository)");
  expect(catalogTable).not.toContain('icon="ph:arrow-square-out"');
  expect(catalogTable).not.toContain('key: "summary"');
  expect(catalogTable).not.toContain("row.summary");
  expect(catalogTable).toContain("return repository;");
  expect(catalogTable).not.toContain("https://git.kimo.ink/agents/");
});

test("扩展库沿用智能体工作区的分类表格与单层操作表面", () => {
  const page = source("pages/agents.vue");
  const sidebar = source("components/agents/AgentSidebar.vue");
  const workspace = source("components/agents/AgentWorkspaceContent.vue");
  const detailDrawer = source(
    "components/extensions/ExtensionDetailDrawer.vue",
  );
  const installModal = source(
    "components/extensions/ExtensionInstallModal.vue",
  );
  const catalog = source("composables/useExtensionCatalog.ts");

  expect(sidebar).toContain("扩展库");
  expect(workspace).toContain("<ExtensionCatalogTable");
  expect(page).toContain("<ExtensionDetailDrawer");
  expect(page).toContain("<ExtensionInstallModal");
  expect(page).toContain('value: "rule"');
  expect(page).toContain('value: "plugin"');
  expect(page).toContain('value: "skill"');
  expect(catalog).not.toContain("mcp:");
  expect(source("stores/relay.ts")).toContain(
    'export type ExtensionCatalogPackage = Omit<ExtensionPackage, "kind"> & {',
  );
  expect(source("stores/relay.ts")).toContain(
    "packages: ExtensionCatalogPackage[];",
  );
  expect(page).toContain('type AgentWorkspace = AgentClient | "extensions";');
  expect(sidebar).toContain(':active="activeWorkspace === client.client"');
  expect(sidebar).toContain(":active=\"activeWorkspace === 'extensions'\"");
  expect(page).toMatch(
    /function selectExtensionCatalog\(\) \{\s+activeWorkspace\.value = "extensions";\s+void extensionCatalog\.load\(activeExtensionSection\.value\);/,
  );
  expect(page).toContain("watch(activeExtensionSection");
  expect(page).toMatch(
    /watch\(activeExtensionSection, \(kind\) => \{\s+if \(activeWorkspace\.value === "extensions"\)\s+void extensionCatalog\.load\(kind, true\);/,
  );
  expect(page).toContain(
    "extensionCatalog.catalogs.value[activeExtensionSection.value].packages",
  );
  expect(page).not.toContain(
    "extensionCatalog.packages(activeExtensionSection.value)",
  );
  expect(page).toContain(
    "extensionCatalog.loading.value[activeExtensionSection]",
  );
  expect(page).not.toContain("showExtensionCatalog");
  expect(page).not.toContain("<small>agents</small>");
  expect(detailDrawer).toContain("MarkdownViewer");
  expect(detailDrawer).toContain('title="扩展详情"');
  expect(detailDrawer).toContain("<template #footer>");
  expect(detailDrawer).toContain(">关闭</Button>");
  expect(detailDrawer).not.toContain("extension-detail__meta");
  expect(detailDrawer).not.toContain("risk");
  expect(detailDrawer).not.toContain(':show-footer="false"');
  expect(detailDrawer).toContain(".extension-detail__readme :deep(h2)");
  expect(detailDrawer).toContain(".extension-detail__readme :deep(p)");
  expect(detailDrawer).not.toContain("white-space: pre-wrap");
  expect(installModal).toContain("<Select");
  expect(installModal).toContain(
    ":title=\"extension ? `安装 ${extension.name}` : '安装扩展'\"",
  );
  expect(installModal).toContain('label="安装到智能体"');
  expect(installModal).toContain('placeholder="选择智能体"');
  expect(installModal).toContain("multiple");
  expect(installModal).toContain("synchronizeExtensionInstallSelection");
  expect(installModal).toContain("function selectClients");
  expect(installModal).toContain("extension_target_exists");
  expect(installModal).toContain('confirmText: "覆盖"');
  expect(installModal).toContain("overwrite");
  expect(installModal).not.toContain("Checkbox");
  expect(installModal).not.toContain("extension_install_preview");
  expect(installModal).not.toContain("正在生成安装变更");
});
