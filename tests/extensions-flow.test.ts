import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("扩展表格显示名称、版本和可复制的仓库地址", () => {
  const catalogTable = source(
    "components/extensions/ExtensionCatalogTable.vue",
  );
  const installationScope = source(
    "components/extensions/ExtensionInstallationScope.vue",
  );
  const clientAvatarGroup = source(
    "components/agents/AgentClientAvatarGroup.vue",
  );

  expect(catalogTable).toContain("function repositoryUrl");
  expect(catalogTable).toContain("function copyRepositoryUrl");
  expect(catalogTable).toContain('key: "name", title: "名称"');
  expect(catalogTable).toContain('key: "source", title: "仓库地址"');
  expect(catalogTable).toContain('key: "installation", title: "安装状态"');
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
  expect(catalogTable).toContain('row.installAction === "update"');
  expect(catalogTable).toContain('row.installAction === "partial"');
  expect(catalogTable).toContain('row.installAction === "installed"');
  expect(catalogTable).toContain('icon: "ph:arrow-circle-down"');
  expect(catalogTable).toContain('label: "更新"');
  expect(catalogTable).toContain(':icon="installAction(row).icon"');
  expect(catalogTable).toContain("<ExtensionInstallationScope");
  expect(
    catalogTable.indexOf('key: "installation", title: "安装状态"'),
  ).toBeLessThan(catalogTable.indexOf('key: "source", title: "仓库地址"'));
  expect(installationScope).toContain("AgentClientAvatarGroup");
  expect(installationScope).toContain("package.installedClients");
  expect(clientAvatarGroup).toContain("AvatarGroup");
  expect(clientAvatarGroup).toContain('variant="capsule"');
  expect(clientAvatarGroup).toContain(':max="4"');
  expect(clientAvatarGroup).toContain('size="small"');
  expect(clientAvatarGroup).toContain(":deep(img)");
  expect(clientAvatarGroup).toContain("var(--pr-monochrome-icon-filter)");
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
  const updates = source("composables/useExtensionUpdates.ts");

  expect(updates).toContain('import { useNotification } from "@stellar/ui";');
  expect(sidebar).toContain("扩展库");
  expect(sidebar).toContain("extensionUpdates");
  expect(sidebar).toContain('semantic="error"');
  expect(workspace).toContain("<ExtensionCatalogTable");
  expect(workspace).toContain('class="extension-update-hint"');
  expect(workspace).toContain("可更新");
  expect(workspace).toContain(
    "v-if=\"activeExtensionSection === 'skill' && extensionUpdates > 0\"",
  );
  expect(workspace).not.toContain(
    'semantic="primary"\n            variant="solid"\n            icon="ph:arrows-clockwise"',
  );
  expect(workspace).toContain(".extension-update-hint");
  expect(workspace).toContain("white-space: nowrap");
  expect(workspace).toContain('"更新中..." : "全部更新"');
  expect(workspace).toContain('icon="ph:arrow-circle-down"');
  expect(workspace).toContain("@click=\"emit('updateAll')\"");
  expect(page).toContain("<ExtensionDetailDrawer");
  expect(page).toContain("<ExtensionInstallModal");
  expect(page).toContain('@refreshed="extensionActions.refresh"');
  expect(page).toContain('value: "rule"');
  expect(page).toContain('value: "skill"');
  expect(page).toContain('value: "mcp"');
  expect(page).not.toContain('value: "plugin"');
  expect(catalog).toContain("mcp: emptyCatalog()");
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
  expect(page).toContain(
    '{ value: "skill", label: "Skill", icon: "ph:book-open-text" }',
  );
  expect(page).toContain(
    '{ value: "mcp", label: "MCP", icon: "ph:plugs-connected" }',
  );
  expect(page).toContain('void extensionCatalog.load("skill")');
  expect(updates).toContain('"extensions_update_all"');
  expect(page).toContain(
    ':extension-updating="extensionActions.updating.value"',
  );
  expect(page).toContain("title: `卸载 ${item.name}`");
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
  expect(installModal).toContain("const actionLabel");
  expect(installModal).toContain('props.extension?.installAction === "update"');
  expect(installModal).toContain(
    'selectedClients.value = ["partial", "update"].includes(',
  );
  expect(installModal).toContain(
    ":title=\"extension ? `${actionLabel} ${extension.name}` : '安装扩展'\"",
  );
  expect(installModal).toContain('label="安装到智能体"');
  expect(installModal).toContain('placeholder="选择智能体"');
  expect(installModal).toContain("multiple");
  expect(installModal).toContain("synchronizeExtensionInstallSelection");
  expect(installModal).toContain("function selectClients");
  expect(installModal).toContain("extension_target_exists");
  expect(installModal).toContain('confirmText: "覆盖"');
  expect(installModal).toContain("overwrite");
  expect(installModal).toContain("Checkbox");
  expect(installModal).toContain("extensions_mcp_preview");
  expect(installModal).toContain("首次使用时可能下载依赖或启动本地进程。");
  expect(installModal).toContain("mcpRiskAccepted");
  expect(installModal).toContain("mcpPreview");
  expect(installModal).toContain('emit("refreshed", props.extension.kind)');
  expect(updates).toContain("async function refresh");
  expect(installModal).toContain('client.value !== "chatgpt"');
  expect(installModal).toContain("MCP 配置");
  expect(installModal).toContain("MCP 环境变量");
  expect(installModal).toContain("服务名");
  expect(installModal).toContain("mcpPreview.manifest.name");
  expect(installModal).toContain("mcpHeaders");
  expect(installModal).toContain("工作目录");
  expect(installModal).toContain("超时时间");
  expect(installModal).toContain("mcpPreview.manifest.transport.enabled");
  expect(installModal).toContain("var(--text-sm)");
  expect(installModal).toContain("var(--st-warning)");
  expect(installModal).toContain("var(--st-semantic-error)");
  expect(installModal).not.toContain("var(--font-size-sm)");
  expect(installModal).not.toContain("var(--st-text-warning)");
  expect(installModal).not.toContain("var(--st-text-error)");
});
