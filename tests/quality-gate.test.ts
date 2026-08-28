import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
  scripts?: Record<string, string>;
};
const releaseWorkflow = readFileSync(".github/workflows/release.yml", "utf8");

test("客户端定义 lint 与格式检查入口", () => {
  expect(packageJson.scripts?.lint).toBeDefined();
  expect(packageJson.scripts?.["format:check"]).toContain(
    "prettier --check app tests",
  );
});

test("发布验证执行前端质量门禁", () => {
  expect(releaseWorkflow).toContain("bun run lint");
  expect(releaseWorkflow).toContain("bun run format:check");
});
