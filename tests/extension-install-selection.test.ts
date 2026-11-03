import { expect, test } from "bun:test";

import type { ExtensionArtifactSummary } from "../app/types/relay";
import {
  extensionSupportsClient,
  linkedAgentsForExtension,
  synchronizeExtensionInstallSelection,
} from "../app/utils/extension/installSelection";

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
  expect(linkedAgentsForExtension(mcp)).toEqual(["codexCli", "chatgpt"]);
});

test("Skill 联动所有可用智能体", () => {
  expect(linkedAgentsForExtension(skill)).toEqual(["codexCli", "chatgpt"]);
});

test("Hook 联动 Codex CLI 与 ChatGPT", () => {
  expect(linkedAgentsForExtension(hook)).toEqual(["codexCli", "chatgpt"]);
  expect(extensionSupportsClient(hook, "codexCli")).toBe(true);
  expect(extensionSupportsClient(hook, "chatgpt")).toBe(true);
});

test("Skill 与 Hook 同包时 ChatGPT 仍可只安装 Skill", () => {
  expect(linkedAgentsForExtension(skillAndHook)).toEqual([
    "codexCli",
    "chatgpt",
  ]);
  expect(extensionSupportsClient(skillAndHook, "chatgpt")).toBe(true);
});

test("勾选规则的 Codex CLI 会同时勾选已检测的 ChatGPT", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt"],
      artifacts: rule,
      next: ["codexCli"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt"]);
});

test("勾选 MCP 的 Codex CLI 会同时勾选全部已检测智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt"],
      artifacts: mcp,
      next: ["codexCli"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt"]);
});

test("取消 MCP 的共享目标会同时取消全部联动智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt"],
      artifacts: mcp,
      next: ["chatgpt"],
      previous: ["codexCli", "chatgpt"],
    }),
  ).toEqual([]);
});

test("勾选 Skill 的任一智能体会同时勾选全部已检测智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt"],
      artifacts: skill,
      next: ["chatgpt"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt"]);
});

test("勾选 Hook 的任一智能体会同时勾选全部 Codex 宿主", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt"],
      artifacts: hook,
      next: ["chatgpt"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt"]);
});

test("取消 Skill 的任一智能体会取消全部联动智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt"],
      artifacts: skill,
      next: ["chatgpt"],
      previous: ["codexCli", "chatgpt"],
    }),
  ).toEqual([]);
});
