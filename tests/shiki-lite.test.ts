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
