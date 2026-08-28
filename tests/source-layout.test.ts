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
