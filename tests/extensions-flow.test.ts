import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("扩展表格显示名称、版本和可复制的仓库地址", () => {
  const catalogTable = source("components/extensions/library/CatalogTable.vue");
  const installationScope = source("components/extensions/install/Scope.vue");
  const clientAvatarGroup = source("components/agents/client/AvatarGroup.vue");

  expect(catalogTable).toContain("function repositoryUrl");
  expect(catalogTable).toContain("function copyRepositoryUrl");
  expect(catalogTable).toContain('key: "name", title: "名称"');
  expect(catalogTable).toContain('key: "source", title: "仓库地址"');
  expect(catalogTable).toContain('key: "installation", title: "安装状态"');
  expect(catalogTable).toContain("#cell-name");
  expect(catalogTable).toContain("#cell-contents");
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
  expect(catalogTable).toContain("const showContents");
  expect(catalogTable).toContain('key: "contents"');
  expect(catalogTable).toContain("width: CONTENTS_COLUMN_WIDTH");
  expect(catalogTable).toContain("const artifactLabelByKind");
  expect(catalogTable).toContain('skill: "Skill"');
  expect(catalogTable).toContain('mcp: "MCP"');
  expect(catalogTable).toContain('hook: "Hook"');
  expect(catalogTable).toContain(
    '<TagGroup :items="artifactLabels(row.artifacts)" />',
  );
  expect(catalogTable).not.toContain("extension-catalog-artifacts");
  expect(catalogTable).not.toContain("artifact.kind.toUpperCase()");
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
  const sidebar = source("components/agents/workspace/Sidebar.vue");
  const workspace = source("components/extensions/library/Content.vue");
  const workspaceStyles = source("components/shell/layout/WorkspaceLayout.vue");
  const selection = source("composables/agent/pageSelection.ts");
  const library = source("composables/extension/library.ts");
  const detailDrawer = source("components/extensions/DetailDrawer.vue");
  const installDrawer = source("components/extensions/install/Drawer.vue");
  const catalog = source("composables/extension/catalog.ts");
  const updates = source("composables/extension/updates.ts");
  const agentClient = source("utils/agent/client.ts");

  expect(updates).toContain('import { useNotification } from "@stellar/ui";');
  expect(sidebar).toContain("扩展库");
  expect(sidebar).toContain("totalExtensionUpdates");
  expect(sidebar).toContain('semantic="error"');
  expect(workspace).toContain("<ExtensionCatalogTable");
  expect(workspace).not.toContain("installTest");
  expect(page).not.toContain("extensions_mcp_test_package");
  expect(workspace).toContain('class="workspace-update-hint"');
  expect(workspace).toContain("可更新");
  expect(workspace).toContain('v-if="updates > 0"');
  expect(workspace).toContain("@click=\"emit('updateAll')\"");
  expect(workspace).not.toContain(
    'semantic="primary"\n            variant="solid"\n            icon="ph:arrows-clockwise"',
  );
  expect(workspaceStyles).toContain(".workspace-update-hint");
  expect(workspaceStyles).toContain("white-space: nowrap");
  expect(workspace).toContain('"更新中..." : "全部更新"');
  expect(workspace).toContain('icon="ph:arrow-circle-down"');
  expect(workspace).toContain("@click=\"emit('updateAll')\"");
  expect(page).toContain("<ExtensionDetailDrawer");
  expect(page).toContain("<ExtensionInstallDrawer");
  expect(selection).toContain("agentSectionOptions");
  expect(page).not.toContain("ph:plugs-connected");
  expect(page).toContain('@refreshed="refreshExtensions"');
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
  expect(workspace).toContain('value: "plugin"');
  expect(catalog).toContain("plugin: emptyCatalog()");
  expect(source("types/relay.ts")).toContain(
    "export type ExtensionCatalogPackage = ExtensionPackage;",
  );
  expect(source("types/relay.ts")).toContain(
    "packages: ExtensionCatalogPackage[];",
  );
  expect(agentClient).toContain(
    'export type AgentWorkspace = AgentClient | "extensions";',
  );
  expect(sidebar).toContain(':active="activeWorkspace === client.client"');
  expect(sidebar).toContain(":active=\"activeWorkspace === 'extensions'\"");
  expect(selection).toContain("function activateExtensions()");
  expect(selection).toContain('activeWorkspace.value = "extensions";');
  expect(library).toContain("watch(activeSection");
  expect(library).toContain("catalog.load(kind, true)");
  expect(library).toContain(
    "catalog.catalogs.value[activeSection.value].packages",
  );
  expect(library).not.toContain("catalog.packages(activeSection.value)");
  expect(library).toContain("catalog.loading.value[activeSection.value]");
  expect(library).toContain(
    "extensionCatalogKinds.map((kind) => catalog.load(kind))",
  );
  expect(updates).toContain('"extensions_update_all"');
  expect(page).toContain(':updating="extensionUpdating"');
  expect(source("composables/agent/itemActions.ts")).toContain(
    "agentUninstallTitle(item)",
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
  expect(installDrawer).toContain("props.extension?.version");
  expect((installDrawer.match(/<Tag/g) ?? []).length).toBe(1);
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
  const mcpInstallForm = source("components/extensions/install/McpForm.vue");
  const previewComposable = source("composables/extension/installPreview.ts");
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
  expect(previewComposable).toContain("extensions_mcp_preview");
  expect(installDrawer).toContain("mcpPreview");
  expect(installDrawer).toContain("useExtensionInstallPreview");
  expect(previewComposable).not.toContain("extensions_skill_preview");
  expect(previewComposable).not.toContain("extensions_hook_preview");
  expect(installDrawer).not.toContain("ExtensionContentsList");
  expect(installDrawer).not.toContain("skillPreview");
  expect(installDrawer).not.toContain("hookPreview");
  expect(installDrawer).not.toContain("hasContents");
  expect(installDrawer).not.toContain("contentsLoading");
  expect(installDrawer).toContain(
    'emit("refreshed", props.extension.catalogKind)',
  );
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

test("扩展库省略就绪状态时仍显示工具栏和目录", () => {
  const workspace = source("components/shell/layout/WorkspaceLayout.vue");

  expect(workspace).toContain("withDefaults(");
  expect(workspace).toContain("ready: true");
  expect(workspace).toContain('v-if="ready === false"');
});

test("扩展库只负责安装，Skill 按包展示且整包卸载挂在智能体条目上", () => {
  const catalogTable = source("components/extensions/library/CatalogTable.vue");
  const workspace = source("components/extensions/library/Content.vue");
  const pluginList = source("components/agents/workspace/items/PluginList.vue");
  const hookList = source("components/agents/workspace/items/HookList.vue");
  const uninstall = source("utils/agent/uninstall.ts");
  const localCommand = source("composables/command/local.ts");
  const nativeItems = readFileSync(
    new URL("../src-tauri/src/agents/items.rs", import.meta.url),
    "utf8",
  );
  const nativeSkillScan = readFileSync(
    new URL("../src-tauri/src/agents/items/skill_scan.rs", import.meta.url),
    "utf8",
  );
  const nativeHooks = readFileSync(
    new URL("../src-tauri/src/extensions/hooks.rs", import.meta.url),
    "utf8",
  );

  expect(catalogTable).not.toContain("canUninstall");
  expect(catalogTable).not.toContain("uninstall");
  expect(workspace).not.toContain("uninstallExtension");
  expect(localCommand).not.toContain("extensions_uninstall");
  expect(localCommand).not.toContain("extensions_hook_preview");
  expect(localCommand).not.toContain("extensions_skill_preview");
  expect(uninstall).toContain("该条目属于扩展包");
  expect(source("composables/agent/itemActions.ts")).toContain(
    "agentUninstallDescription(item)",
  );
  expect(pluginList).toContain("item.artifacts");
  expect(pluginList).toContain("agent-artifact-lines");
  expect(hookList).toContain("const groups = computed");
  expect(hookList).toContain("hookMatcher(item) || '未限定'");
  expect(hookList).toContain("item.command");
  expect(nativeSkillScan).toContain("fn managed_skill_owner");
  expect(nativeItems).toContain("uninstall_plugin_for_client");
  expect(nativeHooks).not.toContain("command_file_name");
  expect(nativeHooks).toContain("name: hook.event");
  expect(nativeHooks).toContain("command: Some(hook.command)");
});

test("扩展更新检查跟随进程，前端只订阅结果并在侧栏提示", () => {
  const app = source("app.vue");
  const stream = source("composables/extension/updateStream.ts");
  const catalog = source("composables/extension/catalog.ts");
  const sidebar = source("components/agents/workspace/Sidebar.vue");
  const shell = source("components/dashboard/frame/Shell.vue");
  const nativeChecks = readFileSync(
    new URL("../src-tauri/src/extensions/checks.rs", import.meta.url),
    "utf8",
  );

  expect(nativeChecks).toContain("tokio::time::interval");
  expect(nativeChecks).toContain("UPDATE_CHECK_INTERVAL");
  expect(nativeChecks).toContain("extensions:catalog");
  expect(nativeChecks).toContain(
    "app.emit(CATALOG_EVENT, CatalogEvent { kind, snapshot })",
  );
  expect(nativeChecks).toContain("CatalogKind::Rule");
  expect(nativeChecks).toContain("CatalogKind::Plugin");
  expect(nativeChecks).toContain("spawn_update_checks");
  expect(app).toContain("useExtensionUpdateStream");
  expect(app).toContain("void extensionStream.start();");
  expect(app).toContain("extensionStream.stop();");
  expect(app.indexOf("void extensionStream.start();")).toBeLessThan(
    app.indexOf("unlistenTraySettings = await listen"),
  );
  expect(stream).toContain("listen<CatalogEvent>");
  expect(stream).toContain("extensionCatalog.applySnapshot(");
  expect(stream).toContain("event.payload.kind");
  expect(stream).toContain("EXTENSION_CATALOG_EVENT");
  expect(stream).toContain("applySnapshot");
  expect(stream).not.toContain("setInterval");
  expect(catalog).toContain("function applySnapshot");
  expect(catalog).toContain("totalUpdateCount");
  expect(catalog).toMatch(/"rule",\s*"plugin",?/);
  expect(sidebar).not.toContain("dot");
  expect(sidebar).toContain(':count="totalExtensionUpdates"');
  expect(sidebar).not.toContain(":max=");
  expect(sidebar).not.toContain("agent-extension-notice");
  expect(shell).toContain("useExtensionCatalog().totalUpdateCount");
  expect(shell).toContain("#badge");
  expect(shell).toContain(
    "v-if=\"item.path === '/agents' && totalExtensionUpdates > 0\"",
  );
  expect(shell).toContain("dashboard-nav-notice");
  expect(shell).toContain("dot");
  expect(shell).toContain('size="small"');
});
