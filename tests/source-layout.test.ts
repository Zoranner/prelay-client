import { expect, test } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const sourceRoots = ["app", "tests", "src-tauri/src", "src-tauri/tests"];
const sourceExtensions = new Set([".js", ".rs", ".ts", ".vue"]);
const maxSourceLines = 450;

function sourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    return sourceExtensions.has(path.slice(path.lastIndexOf(".")))
      ? [path]
      : [];
  });
}

test("客户端源文件不超过 450 行", () => {
  const oversized = sourceRoots
    .flatMap(sourceFiles)
    .map((path) => ({
      path,
      lines: readFileSync(path, "utf8").split(/\r?\n/).length,
    }))
    .filter(({ lines }) => lines > maxSourceLines);

  expect(oversized).toEqual([]);
});

test("前端目录不保留单文件包装层", () => {
  const roots = [
    "app/components",
    "app/utils",
    "app/composables",
    "app/stores",
    "app/types",
  ];
  const singleEntryDirectories = roots.flatMap((root) => {
    const directories = [root];
    const result: Array<{ path: string; entries: string[] }> = [];
    while (directories.length > 0) {
      const path = directories.pop();
      if (!path) continue;
      const entries = readdirSync(path, { withFileTypes: true });
      if (entries.length < 2) {
        result.push({
          path,
          entries: entries.map((entry) => entry.name),
        });
      }
      directories.push(
        ...entries
          .filter((entry) => entry.isDirectory())
          .map((entry) => join(path, entry.name)),
      );
    }
    return result;
  });

  expect(singleEntryDirectories).toEqual([]);
});

test("每个组合式模块都导出 use 组合式函数", () => {
  const missingComposable = sourceFiles("app/composables")
    .filter(
      (path) =>
        !/export\s+(?:async\s+)?function\s+use\w+/.test(
          readFileSync(path, "utf8"),
        ),
    )
    .map((path) => path);

  expect(missingComposable).toEqual([]);
});
