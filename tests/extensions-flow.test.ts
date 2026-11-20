import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("扩展表格显示名称、版本和可复制的仓库地址", () => {
  const catalogTable = source("components/extensions/CatalogTable.vue");
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
  expect(catalogTable).toContain("installAction(row).icon");
  expect(catalogTable).toContain("installingJob(row)");
  expect(catalogTable).toContain("ph:circle-notch");
  expect(catalogTable).toContain("ph:clock");
  expect(catalogTable).toContain(".iconify");
  expect(catalogTable).toContain('semantic: "primary" as const');
  expect(catalogTable).toContain(
    'extension.installAction === "update" ? 0 : 1',
  );
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

test("扩展页独立成页并沿用统一分类表格与单层操作表面", () => {
  const page = source("pages/extensions.vue");
  const agentsPage = source("pages/agents.vue");
  const sidebar = source("components/agents/workspace/Sidebar.vue");
  const shell = source("components/dashboard/frame/Shell.vue");
  const selection = source("composables/agent/pageSelection.ts");
  const library = source("composables/extension/library.ts");
  const installJobs = source("composables/extension/installJobs.ts");
  const detailDrawer = source("components/extensions/DetailDrawer.vue");
  const installDrawer = source("components/extensions/install/Drawer.vue");
  const catalog = source("composables/extension/catalog.ts");
  const updates = source("composables/extension/updates.ts");
  const agentClient = source("utils/agent/client.ts");

  expect(shell).toContain(
    '{ label: "扩展", path: "/extensions", icon: "ph:storefront" }',
  );
  expect(shell).toContain(
    "v-if=\"item.path === '/extensions' && totalExtensionUpdates > 0\"",
  );
  expect(sidebar).not.toContain("扩展");
  expect(sidebar).not.toContain("totalExtensionUpdates");
  expect(page).toContain("<ExtensionCatalogTable");
  expect(page).not.toContain("installTest");
  expect(page).not.toContain("extensions_mcp_test_package");
  expect(page).toContain('v-if="extensionUpdates > 0 || batchUpdating"');
  expect(page).toContain('@click="updateAllExtensions"');
  expect(page).toContain("updateExtensions(extensionPackages.value)");
  expect(page).toContain('"更新中..." : "全部更新"');
  expect(page).toContain('icon="ph:arrow-circle-down"');
  expect(page).not.toContain("可更新");
  expect(page).not.toContain("extension-update-hint");
  const headerActions =
    page.match(/<template #header-actions>[\s\S]*?<\/template>/)?.[0] ?? "";
  expect(headerActions).toContain('semantic="primary"');
  expect(headerActions).toContain('variant="solid"');
  expect(headerActions).toContain('icon="ph:arrows-clockwise"');
  expect(headerActions).toContain('@click="refreshExtensionsPage"');
  expect(headerActions).not.toContain("updateAllExtensions");
  const extensionToolbar =
    page.match(
      /<div class="extension-library__toolbar">[\s\S]*?<\/div>/,
    )?.[0] ?? "";
  expect(extensionToolbar).toContain('@click="updateAllExtensions"');
  expect(extensionToolbar).toContain('icon="ph:arrow-circle-down"');
  expect(extensionToolbar).toContain('class="extension-library__update"');
  expect(extensionToolbar).not.toContain('semantic="primary"');
  expect(page).toMatch(/\.extension-library__update \{[^}]*display: flex/);
  expect(page).toMatch(/\.extension-library__update \{[^}]*flex: 0 0 auto/);
  expect(page).toContain("<ExtensionDetailDrawer");
  expect(page).toContain("<ExtensionInstallDrawer");
  expect(page).toContain("refreshClientStatuses");
  expect(page).toContain("refreshExtensionsPage");
  expect(page).toContain('"刷新中..." : "刷新"');
  expect(page).not.toContain("#header-inline");
  expect(agentsPage).not.toContain("ExtensionLibraryContent");
  expect(agentsPage).not.toContain("<ExtensionInstallDrawer");
  expect(agentsPage).not.toContain("<ExtensionDetailDrawer");
  expect(selection).toContain("agentSectionOptions");
  expect(page).not.toContain("ph:plugs-connected");
  expect(page).toContain("useExtensionInstallJobs");
  expect(page).toContain(':installing="activeJobs"');
  expect(page).toContain('@install="installExtension"');
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
  expect(page).toContain('value: "plugin"');
  expect(catalog).toContain("plugin: emptyCatalog()");
  expect(source("types/relay.ts")).toContain(
    "export type ExtensionCatalogPackage = ExtensionPackage;",
  );
  expect(source("types/relay.ts")).toContain(
    "packages: ExtensionCatalogPackage[];",
  );
  expect(agentClient).not.toContain('"extensions"');
  expect(agentClient).not.toContain("AgentWorkspace");
  expect(sidebar).toContain(':active="activeClient === client.client"');
  expect(selection).not.toContain("activateExtensions");
  expect(selection).not.toContain("activeWorkspace");
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
  expect(installJobs).toContain('"extensions_install"');
  expect(page).toContain(':disabled="batchUpdating"');
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
  expect(installJobs).toContain("extension_target_exists");
  expect(installJobs).toContain('confirmText: "覆盖"');
  expect(installJobs).toContain("overwrite");
  expect(installJobs).toContain("if (jobs.value[name]) return");
  expect(installJobs).toContain("useState<Record<string, InstallJob>>");
  expect(installJobs).toContain("async function updateAll");
  expect(installJobs).toContain("runUpdateQueue(");
  expect(installJobs).toContain('label: "等待中"');
  expect(installJobs).toContain('setJob(extension, "更新中", true)');
  expect(installDrawer).not.toContain("extensions_install");
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
  expect(installDrawer).toContain('emit("install", {');
  expect(updates).toContain("async function refresh");
  expect(installDrawer).not.toContain('client !== "chatgpt"');
  expect(installJobs).toContain("MCP 服务");
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

test("扩展目录已迁出智能体工作区", () => {
  expect(
    existsSync(
      new URL("../app/components/extensions/library", import.meta.url),
    ),
  ).toBe(false);
  expect(
    existsSync(
      new URL("../app/components/extensions/CatalogTable.vue", import.meta.url),
    ),
  ).toBe(true);
  expect(
    existsSync(new URL("../app/pages/extensions.vue", import.meta.url)),
  ).toBe(true);
});

test("扩展页面独立于智能体工作区布局", () => {
  const page = source("pages/extensions.vue");

  expect(page).toContain("<PanelSection");
  expect(page).toContain('<PanelSection title="扩展">');
  expect(page).toContain('class="extension-library__toolbar"');
  expect(page).toContain("<RadioGroup");
  expect(page).not.toContain("#header-inline");
  expect(page).toContain("<template #header-actions>");
  expect(page).not.toContain("WorkspaceLayout");
  expect(page).not.toContain("agent-sidebar");
});

test("扩展只负责安装，Skill 按包展示且整包卸载挂在智能体条目上", () => {
  const catalogTable = source("components/extensions/CatalogTable.vue");
  const page = source("pages/extensions.vue");
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
  expect(page).not.toContain("uninstallExtension");
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

test("扩展更新检查跟随进程，前端只订阅结果并在导航提示", () => {
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
  expect(sidebar).not.toContain("totalExtensionUpdates");
  expect(shell).toContain("useExtensionCatalog().totalUpdateCount");
  expect(shell).toContain("#badge");
  expect(shell).toContain(
    "v-if=\"item.path === '/extensions' && totalExtensionUpdates > 0\"",
  );
  expect(shell).toContain("dashboard-nav-notice");
  expect(shell).toContain("dot");
  expect(shell).toContain('size="small"');
});
