import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";

const appSource = () =>
  readFileSync(new URL("../app/app.vue", import.meta.url), "utf8");

test("client uses Nuxt Tauri and Tailwind entrypoints", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );
  const config = readFileSync(
    new URL("../nuxt.config.ts", import.meta.url),
    "utf8",
  );

  expect(packageJson.scripts.typecheck).toBe("nuxt typecheck");
  expect(packageJson.devDependencies["@tauri-apps/cli"]).toBeDefined();
  expect(config).toContain("@tailwindcss/vite");
  expect(config).toContain('compatibilityDate: "2026-08-19"');
});

test("Tauri uses the static Nuxt output and a fixed development port", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );
  const nuxtConfig = readFileSync(
    new URL("../nuxt.config.ts", import.meta.url),
    "utf8",
  );
  const tauriConfig = JSON.parse(
    readFileSync(
      new URL("../src-tauri/tauri.conf.json", import.meta.url),
      "utf8",
    ),
  );

  expect(packageJson.scripts.generate).toBe("nuxt generate");
  expect(nuxtConfig).toContain("ssr: false");
  expect(nuxtConfig).toContain("devServer:");
  expect(nuxtConfig).toContain("port: 18081");
  expect(nuxtConfig).toContain("strictPort: true");
  expect(tauriConfig.build.devUrl).toBe("http://localhost:18081");
  expect(tauriConfig.build.beforeBuildCommand).toBe("bun run generate");
  expect(tauriConfig.build.frontendDist).toBe("../.output/public");
});

test("管理 API 不可达时由应用根节点显示阻断层", () => {
  const app = appSource();
  expect(app).toContain("useRelayManagementApiStatus");
  expect(app).toContain("managementApi.error");
  expect(app).toContain("Result");
  expect(app).toContain("NotificationContainer");
  expect(app).toContain("重新加载");
});

test("桌面窗口只保留自绘标题栏和内容工作区", () => {
  const tauriConfig = readFileSync(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
    "utf8",
  );
  const titlebar = readFileSync(
    new URL("../app/components/shell/AppTitlebar.vue", import.meta.url),
    "utf8",
  );
  const shell = readFileSync(
    new URL("../app/components/dashboard/DashboardShell.vue", import.meta.url),
    "utf8",
  );
  const statusbar = readFileSync(
    new URL(
      "../app/components/dashboard/DashboardStatusbar.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const capability = readFileSync(
    new URL("../src-tauri/capabilities/default.json", import.meta.url),
    "utf8",
  );
  const styles = readFileSync(
    new URL("../app/assets/css/main.css", import.meta.url),
    "utf8",
  );

  expect(tauriConfig).toContain('"decorations": false');
  expect(appSource()).toContain("AppTitlebar");
  expect(appSource()).toContain("<AppTitlebar />");
  expect(titlebar).toContain("data-tauri-drag-region");
  expect(titlebar).toContain("getCurrentWindow");
  expect(shell).not.toContain("<AppTitlebar");
  expect(shell).toContain("DashboardStatusbar");
  expect(shell).toContain("--pr-statusbar-height");
  expect(styles).toContain("--pr-statusbar-height: var(--height-status);");
  expect(capability).toContain('"core:window:allow-start-dragging"');
  expect(shell).toContain('<Sidebar variant="rail" :show-header="false">');
  expect(statusbar).toContain("dashboard-statusbar");
  expect(statusbar).toContain("getVersion");
  expect(statusbar).toContain("managementApi.error");
  expect(statusbar).toContain("ph:arrows-left-right");
  expect(statusbar).toContain('navigateTo("/setup?change=1")');
  expect(statusbar).toContain("<Button");
  expect(statusbar).toContain('size="tiny"');
  expect(statusbar).toContain("clientUpdate.check()");
  expect(statusbar).toContain("clientUpdate.download()");
  expect(statusbar).toContain("clientUpdate.openInstallDialog()");
});

test("客户端更新在检查、下载和待安装之间保持明确状态", () => {
  const update = readFileSync(
    new URL("../app/composables/useClientUpdate.ts", import.meta.url),
    "utf8",
  );
  const dialog = readFileSync(
    new URL(
      "../app/components/settings/ClientUpdateDialog.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(update).toMatch(
    /"idle"[\s|]+"checking"[\s|]+"available"[\s|]+"downloading"[\s|]+"ready"/,
  );
  expect(update).toContain('"client_update_prepare"');
  expect(update).toContain('invoke("client_update_prepare", {');
  expect(update).toContain('state.value = "downloading"');
  expect(update).toContain('state.value = "ready"');
  expect(dialog).toContain('title="新版本已就绪"');
  expect(dialog).toContain("稍后安装");
  expect(dialog).toMatch(
    /Prelay\s+\{\{\s+version\s+\}\}\s+已下载，安装时将短暂退出，不影响其他智能体和正在进行的对话及调用。/,
  );
  expect(dialog).not.toContain("client-update-dialog__notice");
});

test("桌面客户端启动后自动下载可用更新并提示安装", () => {
  const update = readFileSync(
    new URL("../app/composables/useClientUpdate.ts", import.meta.url),
    "utf8",
  );

  expect(appSource()).toContain("clientUpdate.checkAndDownload()");
  expect(update).toContain("async function checkAndDownload()");
  expect(update).toContain("await check()");
  expect(update).toContain('if (state.value === "available") await download()');
  expect(update).toContain("checkAndDownload,");
});

test("仪表盘趋势图按需加载 ECharts", () => {
  const chart = readFileSync(
    new URL(
      "../app/components/dashboard/TokenUsageTrendChart.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(chart).toContain('await import("echarts")');
  expect(chart).not.toContain('import * as echarts from "echarts"');
});

test("仪表盘趋势图将输出堆叠在输入之上", () => {
  const chart = readFileSync(
    new URL(
      "../app/components/dashboard/TokenUsageTrendChart.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(chart).toMatch(
    /name: "输入"[\s\S]{0,180}stack: "token-usage"[\s\S]{0,180}name: "输出"[\s\S]{0,180}stack: "token-usage"/,
  );
});

test("桌面和网页标题栏复用 Prelay 图标资产", () => {
  const tauriConfig = JSON.parse(
    readFileSync(
      new URL("../src-tauri/tauri.conf.json", import.meta.url),
      "utf8",
    ),
  );
  const titlebar = readFileSync(
    new URL("../app/components/shell/AppTitlebar.vue", import.meta.url),
    "utf8",
  );

  expect(tauriConfig.bundle.icon).toEqual(["icons/icon.ico", "icons/icon.png"]);
  expect(
    existsSync(
      new URL("../app/assets/images/prelay-icon.png", import.meta.url),
    ),
  ).toBe(true);
  expect(titlebar).toContain(
    'import prelayIcon from "~/assets/images/prelay-icon.png"',
  );
  expect(titlebar).toContain('<img :src="prelayIcon" alt=""');
  expect(titlebar).not.toContain(">PR</span>");
  expect(titlebar).toMatch(
    /class="window-action-close"[\s\S]{0,120}variant="ghost"/,
  );
});

test("仪表盘头像使用 DiceBear Cutouts 预设", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );
  const shell = readFileSync(
    new URL("../app/components/dashboard/DashboardShell.vue", import.meta.url),
    "utf8",
  );

  expect(packageJson.dependencies["@dicebear/core"]).toBe("10.5.0");
  expect(packageJson.dependencies["@dicebear/styles"]).toBe("10.5.0");
  expect(packageJson.dependencies["@dicebear/identicon"]).toBeUndefined();
  expect(shell).toContain(
    'import cutouts from "@dicebear/styles/cutouts.json"',
  );
  expect(shell).toContain("new Style(cutouts)");
  expect(shell).toContain("new DiceBearAvatar(cutoutsStyle");
  expect(shell).toContain("bootstrap.value?.identity_id");
});

test("托盘设置菜单打开桌面偏好弹窗", () => {
  const app = appSource();

  expect(app).toContain('listen("tray:open-settings"');
  expect(app).toContain("DesktopPreferencesDialog");
  expect(app).toContain("desktopPreferencesDialog.open()");
  expect(app).toContain("unlistenTraySettings");
});

test("全局浮层始终为窗口边框、标题栏和状态栏保留安全区", () => {
  const styles = readFileSync(
    new URL("../app/assets/css/main.css", import.meta.url),
    "utf8",
  );
  const config = readFileSync(
    new URL("../nuxt.config.ts", import.meta.url),
    "utf8",
  );

  expect(appSource()).not.toContain("pr-desktop-shell");
  expect(styles).not.toContain('@import "@stellar/ui/styles"');
  expect(config).toContain('"@stellar/ui/nuxt"');
  expect(config).not.toContain("modules:done");
  expect(styles).toMatch(
    /:root\s*\{[\s\S]*--st-overlay-inset:\s*calc\(var\(--pr-titlebar-height\) \+ 1px\) 1px\s+calc\(var\(--pr-statusbar-height\) \+ 1px\) 1px;/,
  );
  expect(styles).not.toContain(".pr-desktop-shell");
});

test("客户端仅使用组件库已定义的主题令牌", () => {
  const styles = readFileSync(
    new URL("../app/assets/css/main.css", import.meta.url),
    "utf8",
  );
  const diagnostics = readFileSync(
    new URL("../app/components/activity/RequestTable.vue", import.meta.url),
    "utf8",
  );
  const agents = readFileSync(
    new URL("../app/components/agents/AgentItemList.vue", import.meta.url),
    "utf8",
  );
  const statusbar = readFileSync(
    new URL(
      "../app/components/dashboard/DashboardStatusbar.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(styles).not.toContain("--pr-dashboard-gap");
  expect(diagnostics).not.toContain("--st-color-warning");
  expect(agents).not.toContain("--st-text-danger");
  expect(statusbar).not.toContain("--st-border-focus");
});

test("设置入口打开桌面偏好弹窗", () => {
  const titlebar = readFileSync(
    new URL("../app/components/shell/AppTitlebar.vue", import.meta.url),
    "utf8",
  );
  const preferences = readFileSync(
    new URL("../app/composables/useDesktopPreferences.ts", import.meta.url),
    "utf8",
  );

  expect(preferences).toContain('"desktop_preferences_get"');
  expect(preferences).toContain('"desktop_preferences_save"');
  expect(preferences).toContain("applyTheme,");
  expect(titlebar).toContain("useDesktopPreferencesDialog");
  expect(titlebar).toContain("openDesktopPreferences");
  expect(titlebar).toMatch(
    /icon="ph:gear-six"[\s\S]{0,240}aria-label="设置"[\s\S]{0,240}@click="openDesktopPreferences"[\s\S]{0,240}icon="ph:minus"/,
  );
});
