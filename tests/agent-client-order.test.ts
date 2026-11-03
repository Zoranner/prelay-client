import { expect, test } from "bun:test";

import { sortAgentClients } from "../app/utils/agent/client";

test("已安装的智能体排在未安装智能体之前", () => {
  const clients = sortAgentClients([
    { client: "chatgpt" as const, installed: false },
    { client: "codexCli" as const, installed: true },
  ]);

  expect(clients.map(({ client }) => client)).toEqual(["codexCli", "chatgpt"]);
});

test("已安装的智能体按 ChatGPT、Codex CLI 排序", () => {
  const clients = sortAgentClients([
    { client: "codexCli" as const, installed: true },
    { client: "chatgpt" as const, installed: true },
  ]);

  expect(clients.map(({ client }) => client)).toEqual(["chatgpt", "codexCli"]);
});
