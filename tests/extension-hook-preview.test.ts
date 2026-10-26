import { expect, test } from "bun:test";
import {
  hookPreviewMatchesExtension,
  type HookInstallPreview,
} from "~/utils/extensionHookPreview";
import type { ExtensionCatalogPackage } from "~/stores/relay";

const extension: ExtensionCatalogPackage = {
  name: "superpowers",
  repository: "https://git.example.test/agents/superpowers",
  commitSha: "expected-commit",
  version: "v1.0.0",
  catalogKind: "plugin",
  artifacts: [{ kind: "hook", fileCount: 2 }],
  installAction: "install",
  installedClients: [],
};

const preview: HookInstallPreview = {
  name: "superpowers",
  version: "v1.0.0",
  commitSha: "expected-commit",
  files: ["hooks/hooks.json", "hooks/scripts/session-start.sh"],
  review: [
    {
      event: "SessionStart",
      matcher: "startup",
      command:
        "sh /home/test/.prelay/packages/superpowers/commit/hooks/scripts/session-start.sh",
      commandWindows: null,
    },
  ],
};

test("Hook 预览必须匹配当前扩展的固定版本", () => {
  expect(hookPreviewMatchesExtension(preview, extension)).toBe(true);
  expect(
    hookPreviewMatchesExtension(preview, {
      ...extension,
      commitSha: "moved-tag-commit",
    }),
  ).toBe(false);
});
