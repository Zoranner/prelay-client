import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

test("必装包显示标记并只对未受管内容弹覆盖确认", () => {
  const table = readFileSync(
    new URL("../app/components/extensions/CatalogTable.vue", import.meta.url),
    "utf8",
  );
  const jobs = readFileSync(
    new URL("../app/composables/extension/installJobs.ts", import.meta.url),
    "utf8",
  );
  const updates = readFileSync(
    new URL("../app/composables/extension/updates.ts", import.meta.url),
    "utf8",
  );
  const stream = readFileSync(
    new URL("../app/composables/extension/updateStream.ts", import.meta.url),
    "utf8",
  );
  const feedback = readFileSync(
    new URL("../app/composables/extension/policyFeedback.ts", import.meta.url),
    "utf8",
  );

  expect(table).toContain("row.policy === 'required'");
  expect(table).toContain("必装");
  expect(jobs).toContain('error.code !== "extension_target_exists"');
  expect(jobs).toContain("local_extensions_conflict");
  expect(updates).toContain("extensions_policy_sync");
  expect(stream).toContain("extensions:policy");
  expect(stream).toContain("event.payload");
  expect(feedback).toContain("扩展策略同步失败");
  expect(feedback).toContain("扩展策略同步警告");
});
