import { expect, test } from "bun:test";

import {
  linkedAgentsForExtension,
  synchronizeExtensionInstallSelection,
} from "../app/utils/extensionInstallSelection";

test("规则只联动 Codex CLI 与 ChatGPT", () => {
  expect(linkedAgentsForExtension("rule")).toEqual(["codexCli", "chatgpt"]);
});

test("Skill 联动全部智能体", () => {
  expect(linkedAgentsForExtension("skill")).toEqual([
    "codexCli",
    "chatgpt",
    "openCode",
  ]);
});

test("勾选规则的 Codex CLI 会同时勾选已检测的 ChatGPT", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      kind: "rule",
      next: ["codexCli"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt"]);
});

test("勾选 Skill 的任一智能体会同时勾选全部已检测智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      kind: "skill",
      next: ["openCode"],
      previous: [],
    }),
  ).toEqual(["codexCli", "chatgpt", "openCode"]);
});

test("取消 Skill 的任一智能体会取消全部联动智能体", () => {
  expect(
    synchronizeExtensionInstallSelection({
      detected: ["codexCli", "chatgpt", "openCode"],
      kind: "skill",
      next: ["codexCli", "chatgpt"],
      previous: ["codexCli", "chatgpt", "openCode"],
    }),
  ).toEqual([]);
});
