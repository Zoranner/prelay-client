import { expect, test } from "bun:test";

import type { ExtensionArtifactSummary } from "../app/stores/relay";
import {
  extensionSupportsClient,
  linkedAgentsForExtension,
  synchronizeExtensionInstallSelection,
} from "../app/utils/extensionInstallSelection";

const rule: ExtensionArtifactSummary[] = [];
const mcp: ExtensionArtifactSummary[] = [{ kind: "mcp", fileCount: 1 }];
const skill: ExtensionArtifactSummary[] = [{ kind: "skill", fileCount: 2 }];
const hook: ExtensionArtifactSummary[] = [{ kind: "hook", fileCount: 1 }];
const skillAndHook: ExtensionArtifactSummary[] = [
  { kind: "skill", fileCount: 2 },
  { kind: "hook", fileCount: 1 },
];

test("规则只联动 Codex CLI 与 ChatGPT", () => {
  expect(linkedAgentsForExtension(rule)).toEqual(["codexCli", "chatgpt"]);
});

test("MCP 联动所有可用智能体", () => {
  expect(linkedAgentsForExtension(mcp)).toEqual([
    "codexCli",
    "chatgpt",
    "openCode",
  ]);
});

test("Skill 联动所有可用智能体", () => {
  expect(linkedAgentsForExtension(skill)).toEqual([
    "codexCli",
    "chatgpt",
    "openCode",
  ]);
});

test("Hook 只支持 Codex CLI", () => {
  expect(linkedAgentsForExtension(hook)).toEqual(["codexCli"]);
  expect(extensionSupportsClient(hook, "codexCli")).toBe(true);
  expect(extensionSupportsClient(hook, "chatgpt")).toBe(false);
  expect(extensionSupportsClient(hook, "openCode")).toBe(false);
});

test("Skill 与 Hook 同包时 OpenCode 仍可只安装 Skill", () => {
  expect(linkedAgentsForExtension(skillAndHook)).toEqual([
    "codexCli",
    "chatgpt",
    "openCode",
  ]);
  expect(extensionSupportsClient(skillAndHook, "openCode")).toBe(true);
});

test("勾选规则的 Codex CLI 会同时勾选已检测的 ChatGPT", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      artifacts: rule,
      next: ["codexCli"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt"]);
});

test("勾选 MCP 的 Codex CLI 会同时勾选全部已检测智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      artifacts: mcp,
      next: ["codexCli"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt", "openCode"]);
});

test("取消 MCP 的共享目标会同时取消全部联动智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      artifacts: mcp,
      next: ["chatgpt"],
      previous: ["codexCli", "chatgpt", "openCode"],
    }),
  ).toEqual([]);
});

test("勾选 Skill 的任一智能体会同时勾选全部已检测智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      artifacts: skill,
      next: ["openCode"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt", "openCode"]);
});

test("取消 Skill 的任一智能体会取消全部联动智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      artifacts: skill,
      next: ["codexCli", "chatgpt"],
      previous: ["codexCli", "chatgpt", "openCode"],
    }),
  ).toEqual([]);
});
