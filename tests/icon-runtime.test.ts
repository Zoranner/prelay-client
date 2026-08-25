import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { renderToString } from "@vue/server-renderer";
import { createSSRApp, h } from "vue";
import { Icon } from "@stellar/ui";

const iconPlugin = new URL(
  "../app/plugins/stellar-icons.client.ts",
  import.meta.url,
);

test("客户端依赖默认离线可用的组件库图标", () => {
  const packageJson = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );

  expect(packageJson.dependencies["@stellar/ui"]).toBe("0.1.2");
  expect(existsSync(iconPlugin)).toBe(false);
});

test("客户端安装的组件库可离线渲染图标", async () => {
  const html = await renderToString(
    createSSRApp({
      render: () => h(Icon, { icon: "ph:gear-six" }),
    }),
  );

  expect(html).toContain("<svg");
});
