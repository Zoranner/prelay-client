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
    new URL("../app/components/workbench/WorkbenchShell.vue", import.meta.url),
    "utf8",
  );
  const statusbar = readFileSync(
    new URL(
      "../app/components/workbench/WorkbenchStatusbar.vue",
      import.meta.url,
    ),
    "utf8",
  );

  expect(tauriConfig).toContain('"decorations": false');
  expect(appSource()).toContain("AppTitlebar");
  expect(appSource()).toContain("<AppTitlebar />");
  expect(titlebar).toContain("data-tauri-drag-region");
  expect(titlebar).toContain("getCurrentWindow");
  expect(shell).not.toContain("<AppTitlebar");
  expect(shell).toContain("WorkbenchStatusbar");
  expect(shell).toContain("--pr-statusbar-height");
  expect(shell).toContain('<Sidebar variant="rail" :show-header="false">');
  expect(statusbar).toContain("workbench-statusbar");
  expect(statusbar).toContain("getVersion");
  expect(statusbar).toContain("managementApi.error");
  expect(statusbar).toContain("ph:arrows-left-right");
  expect(statusbar).toContain('navigateTo("/setup?change=1")');
  expect(statusbar).toContain('class="workbench-statusbar__switch"');
  expect(statusbar).not.toContain("<Button");
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
    existsSync(new URL("../app/assets/images/prelay-icon.png", import.meta.url)),
  ).toBe(true);
  expect(titlebar).toContain('import prelayIcon from "~/assets/images/prelay-icon.png"');
  expect(titlebar).toContain('<img :src="prelayIcon" alt=""');
  expect(titlebar).not.toContain(">PR</span>");
  expect(titlebar).toMatch(
    /class="window-action-close"[\s\S]{0,120}variant="ghost"/,
  );
});

test("托盘设置菜单打开全局设置弹窗", () => {
  const app = appSource();

  expect(app).toContain('listen("tray:open-settings"');
  expect(app).toContain("desktopPreferencesDialog.open");
  expect(app).toContain("unlistenTraySettings");
});

test("仅桌面宿主为全屏遮罩保留窗口边框、标题栏和状态栏", () => {
  const styles = readFileSync(
    new URL("../app/assets/css/main.css", import.meta.url),
    "utf8",
  );

  expect(appSource()).toMatch(
    /document\.documentElement\.classList\.toggle\(\s*"pr-desktop-shell",\s*isDesktopRuntime,\s*\)/,
  );
  expect(styles).toContain("--st-overlay-inset: 0px;");
  expect(styles).toContain(".pr-desktop-shell");
  expect(styles).toMatch(
    /--st-overlay-inset:\s*calc\(var\(--pr-titlebar-height\) \+ 1px\) 1px\s+calc\(var\(--pr-statusbar-height\) \+ 1px\) 1px;/,
  );
});

test("设置入口打开全局桌面偏好弹窗", () => {
  const dialog = readFileSync(
    new URL(
      "../app/components/settings/DesktopPreferencesDialog.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const shell = readFileSync(
    new URL("../app/components/workbench/WorkbenchShell.vue", import.meta.url),
    "utf8",
  );
  const preferences = readFileSync(
    new URL("../app/composables/useDesktopPreferences.ts", import.meta.url),
    "utf8",
  );

  expect(appSource()).toContain("DesktopPreferencesDialog");
  expect(appSource()).toContain("useDesktopPreferencesDialog");
  expect(dialog).toContain('title="设置"');
  expect(preferences).toContain('"desktop_preferences_get"');
  expect(preferences).toContain('"desktop_preferences_save"');
  expect(preferences).toContain("applyTheme,");
  expect(dialog).toContain("外观主题");
  expect(dialog).toContain("开机自启");
  expect(dialog).toContain("静默启动");
  expect(dialog).toContain("最小化到托盘");
  expect(dialog).toContain("<Toggle");
  expect(dialog).not.toContain("<Checkbox");
  expect(dialog).toContain("watch(() => draft.theme");
  expect(dialog).toContain("desktopPreferences.applyTheme");
  expect(shell).toContain("openDesktopPreferences");
  expect(shell).not.toContain('to="/settings"');
});
