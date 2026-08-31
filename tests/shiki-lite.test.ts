import { expect, test } from "bun:test";
import { codeToHtml } from "../app/utils/shikiLite";

test("轻量代码高亮适配器转义代码并保留语言标记", async () => {
  const html = await codeToHtml('<script>alert("x")</script>', {
    lang: "typescript",
  });

  expect(html).toContain('data-language="typescript"');
  expect(html).toContain("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;");
  expect(html).not.toContain("<script>alert");
});

test("轻量 Shiki 适配器为代码块提供逐行 token", async () => {
  const module = await import("../app/utils/shikiLite");
  const codeToTokensBase = (
    module as {
      codeToTokensBase?: (code: string) => Promise<unknown>;
    }
  ).codeToTokensBase;

  expect(codeToTokensBase).toBeTypeOf("function");
  if (!codeToTokensBase) return;

  await expect(
    codeToTokensBase("const ready = true;\nnext()"),
  ).resolves.toEqual([
    [{ content: "const ready = true;", offset: 0 }],
    [{ content: "next()", offset: 1 }],
  ]);
});
