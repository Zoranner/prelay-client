import { expect, test } from "bun:test";
import {
  mcpPreviewMatchesExtension,
  type ExtensionMcpPreview,
} from "~/utils/extensionMcpPreview";
import type { ExtensionCatalogPackage } from "~/stores/relay";

const extension: ExtensionCatalogPackage = {
  name: "filesystem-mcp",
  repository: "https://git.example.test/agents/filesystem-mcp",
  commitSha: "expected-commit",
  version: "v1.0.0",
  kind: "mcp",
  installAction: "install",
  installedClients: [],
};

const preview: ExtensionMcpPreview = {
  name: "filesystem-mcp",
  version: "v1.0.0",
  commitSha: "expected-commit",
  manifest: {
    name: "filesystem",
    transport: {
      type: "stdio",
      command: ["uvx", "mcp-server-filesystem"],
      cwd: null,
      environment: {},
      enabled: true,
      timeoutMs: null,
    },
  },
};

test("MCP 预览必须匹配当前扩展的固定版本", () => {
  expect(mcpPreviewMatchesExtension(preview, extension)).toBe(true);
  expect(
    mcpPreviewMatchesExtension(preview, {
      ...extension,
      commitSha: "moved-tag-commit",
    }),
  ).toBe(false);
});
