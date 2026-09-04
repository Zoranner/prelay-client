import { expect, test } from "bun:test";
import { IconUsageScanner } from "@nuxt/icon/utils";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const iconPlugin = new URL(
  "../app/plugins/stellar-icons.client.ts",
  import.meta.url,
);
const projectRoot = fileURLToPath(new URL("..", import.meta.url));
const clientBundle = new URL(
  "../.nuxt/nuxt-icon-client-bundle.mjs",
  import.meta.url,
);

test("客户端依赖默认离线可用的组件库图标", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );

  expect(packageJson.dependencies["@stellar/ui"]).toBe("0.1.8");
  expect(existsSync(iconPlugin)).toBe(false);
});

test("Nuxt 将应用图标打包到客户端且不使用远程图标服务", () => {
  const nuxtConfig = readFileSync(
    new URL("../nuxt.config.ts", import.meta.url),
    "utf8",
  );

  expect(nuxtConfig).toContain('"@nuxt/icon",');
  expect(nuxtConfig).toContain('"@stellar/ui/nuxt"');
  expect(nuxtConfig).toContain('provider: "none"');
  expect(nuxtConfig).toContain('componentName: "NuxtIcon"');
  expect(nuxtConfig).toContain("clientBundle:");
  expect(nuxtConfig).toContain("scan: true");
  expect(nuxtConfig).not.toContain("icons: [");
  expect(nuxtConfig).not.toContain("stellarStyles");
  expect(nuxtConfig).not.toContain("modules:done");
});

test("Nuxt 启动扫描能够发现宿主源码中的字面量图标", async () => {
  const icons = await new IconUsageScanner(true).scanFiles(projectRoot);

  expect(icons).toContain("ph:sliders-horizontal");
});

test("生成的离线 bundle 同时包含业务与组件库固定图标", () => {
  const bundle = readFileSync(clientBundle, "utf8");

  expect(bundle).toContain("sliders-horizontal");
  expect(bundle).toContain("check-circle");
  expect(bundle).toContain("x-circle");
  expect(bundle).toContain("info");
});

test("客户端将图标渲染责任交给 Nuxt 图标模块", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );

  expect(packageJson.dependencies["@iconify-json/ph"]).toBeDefined();
  expect(packageJson.dependencies["@stellar/ui"]).toBe("0.1.8");
});
