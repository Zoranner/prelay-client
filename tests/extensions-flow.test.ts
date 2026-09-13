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
  const installDrawer = source(
    "components/extensions/ExtensionInstallDrawer.vue",
  );
  const catalog = source("composables/useExtensionCatalog.ts");
  const updates = source("composables/useExtensionUpdates.ts");
  const agentClient = source("utils/agentClient.ts");

  expect(updates).toContain('import { useNotification } from "@stellar/ui";');
  expect(sidebar).toContain("扩展库");
  expect(sidebar).toContain("extensionUpdates");
  expect(sidebar).toContain('semantic="error"');
  expect(workspace).toContain("<ExtensionCatalogTable");
  expect(workspace).not.toContain("installTest");
  expect(page).not.toContain("extensions_mcp_test_package");
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
  expect(page).toContain("<ExtensionInstallDrawer");
  expect(page).toContain("agentSectionOptions");
  expect(page).not.toContain("ph:plugs-connected");
  expect(page).toContain('@refreshed="extensionActions.refresh"');
  expect(page).toContain("agentSectionOptions");
  expect(agentClient).toContain("export const agentSectionOptions");
  expect(agentClient).toContain(
    '{ value: "mcp", label: "MCP", icon: "ph:terminal-window" }',
  );
  expect(agentClient).toContain(
    '{ value: "skill", label: "Skill", icon: "ph:book-open-text" }',
  );
  expect(agentClient.indexOf('value: "mcp"')).toBeLessThan(
    agentClient.indexOf('value: "skill"'),
  );
  expect(page).not.toContain("ph:plugs-connected");
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
  expect(installDrawer).toContain("<Drawer");
  expect(installDrawer).not.toContain("<Modal");
  expect(installDrawer).toContain("const actionLabel");
  expect(installDrawer).toContain(
    'props.extension?.installAction === "update"',
  );
  expect(installDrawer).toContain(
    'selectedClients.value = ["partial", "update"].includes(',
  );
  expect(installDrawer).toContain("`${actionLabel}扩展`");
  expect(installDrawer).toContain('label="安装到智能体"');
  expect(installDrawer).toContain('placeholder="选择智能体"');
  expect(installDrawer).toContain("multiple");
  expect(installDrawer).toContain("synchronizeExtensionInstallSelection");
  expect(installDrawer).toContain("function selectClients");
  expect(installDrawer).toContain("extension_target_exists");
  expect(installDrawer).toContain('confirmText: "覆盖"');
  expect(installDrawer).toContain("overwrite");
  expect(installDrawer).not.toContain("mcpEnvironmentValues");
  expect(installDrawer).not.toContain("environment-values");
  expect(installDrawer).toContain("McpInstallForm");
  expect(installDrawer).toContain("extension-install-drawer__footer");
  expect(installDrawer).toContain("extension-install-drawer__identity");
  expect(installDrawer).toContain("extension-install-drawer__identity-icon");
  expect(installDrawer).toContain("extension-install-drawer__identity-name");
  expect(installDrawer).toContain("extension-install-drawer__identity-version");
  expect(installDrawer).toContain(
    "extension-install-drawer__identity-transport",
  );
  expect(installDrawer).toContain("kindIcon");
  expect(installDrawer).toContain("agentSectionOptions");
  expect(installDrawer).toContain("props.extension?.version");
  expect((installDrawer.match(/<Badge/g) ?? []).length).toBe(1);
  expect(installDrawer).toContain("extension-install-drawer__preview");
  expect(installDrawer).toContain("extension-install-drawer__target");
  expect(installDrawer).toMatch(
    /extension-install-drawer__identity[\s\S]*extension-install-drawer__target[\s\S]*extension-install-drawer__preview[\s\S]*<template #footer>/,
  );
  expect(installDrawer).not.toMatch(/<template #footer>[\s\S]*安装到智能体/);
  expect(installDrawer).toMatch(
    /\.extension-install-drawer__target \{[^}]*flex: 0 0 auto/,
  );
  expect(installDrawer).toMatch(
    /\.extension-install-drawer__preview \{[^}]*overflow-y: auto/,
  );
  const mcpInstallForm = source("components/extensions/McpInstallForm.vue");
  expect(mcpInstallForm).not.toContain("<Badge");
  expect(mcpInstallForm).not.toContain("mcp-install-identity");
  expect(mcpInstallForm).toContain("mcp-install-spec");
  expect(mcpInstallForm).toContain("mcp-install-items");
  expect(mcpInstallForm).toContain("mcp-install-mapping");
  expect(mcpInstallForm).toContain("argumentsList");
  expect(mcpInstallForm).toContain("启动命令");
  expect(mcpInstallForm).toContain("启动参数");
  expect(mcpInstallForm).not.toContain("CodeBlock");
  expect(mcpInstallForm).not.toContain("navigator.clipboard");
  expect(mcpInstallForm).not.toContain("FormField");
  expect(mcpInstallForm).not.toContain('type="password"');
  expect(mcpInstallForm).toContain("mcp-install-environment__hint");
  expect(mcpInstallForm).toContain("取值由本机环境变量提供");
  expect(mcpInstallForm).toContain("mcp-install-environment__name");
  expect(mcpInstallForm).toMatch(
    /mcp-install-environment[\s\S]*grid-template-columns: minmax\(0, 1fr\)/,
  );
  expect(mcpInstallForm).not.toContain("mcp-install-environment-row");
  expect(mcpInstallForm).not.toContain("mcp-install-argument-list");
  expect(mcpInstallForm).not.toContain("mcp-install-command-block");
  expect(installDrawer).not.toContain("mcpPreview.manifest.transport.enabled");
  expect(installDrawer).not.toContain("mcpRiskAccepted");
  expect(installDrawer).not.toContain("首次使用时可能下载依赖或启动本地进程。");
  expect(installDrawer).toContain("extensions_mcp_preview");
  expect(installDrawer).toContain("mcpPreview");
  expect(installDrawer).toContain('emit("refreshed", props.extension.kind)');
  expect(updates).toContain("async function refresh");
  expect(installDrawer).not.toContain('client !== "chatgpt"');
  expect(installDrawer).toContain("MCP 服务");
  expect(installDrawer).toContain("manifest.name");
  expect(installDrawer).toContain("manifest.transport.type");
  expect(mcpInstallForm).toContain("headers");
  expect(mcpInstallForm).not.toContain("工作目录");
  expect(mcpInstallForm).not.toContain("超时时间");
  expect(mcpInstallForm).toContain('isStdio ? "启动配置" : "连接配置"');
  expect(mcpInstallForm).toContain("环境变量");
  expect(mcpInstallForm).toContain("var(--text-sm)");
  expect(installDrawer).toContain("var(--st-semantic-error)");
  expect(installDrawer).not.toContain("var(--font-size-sm)");
  expect(installDrawer).not.toContain("var(--st-text-warning)");
  expect(installDrawer).not.toContain("var(--st-text-error)");
});
