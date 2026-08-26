import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";

const iconPlugin = new URL(
  "../app/plugins/stellar-icons.client.ts",
  import.meta.url,
);

test("客户端依赖默认离线可用的组件库图标", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );

  expect(packageJson.dependencies["@stellar/ui"]).toBe("0.1.3");
  expect(existsSync(iconPlugin)).toBe(false);
});

test("Nuxt 将应用图标打包到客户端且不使用远程图标服务", () => {
  const nuxtConfig = readFileSync(
    new URL("../nuxt.config.ts", import.meta.url),
    "utf8",
  );

  expect(nuxtConfig).toContain('modules: ["@nuxt/icon", "@stellar/ui/nuxt"]');
  expect(nuxtConfig).toContain('provider: "none"');
  expect(nuxtConfig).toContain('componentName: "NuxtIcon"');
  expect(nuxtConfig).toContain("clientBundle:");
  expect(nuxtConfig).toContain("scan: true");
  expect(nuxtConfig).toContain('"ph:arrows-in"');
  expect(nuxtConfig).toContain('"ph:arrows-out"');
  expect(nuxtConfig).toContain('"ph:spinner-gap"');
});

test("客户端将图标渲染责任交给 Nuxt 图标模块", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );

  expect(packageJson.dependencies["@iconify-json/ph"]).toBeDefined();
  expect(packageJson.dependencies["@stellar/ui"]).toBe("0.1.3");
});
