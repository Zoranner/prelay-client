import { expect, test } from "bun:test";

import { sortAgentClients } from "../app/utils/agentClient";

test("已安装的智能体排在未安装智能体之前", () => {
  const clients = sortAgentClients([
    { client: "chatgpt" as const, installed: false },
    { client: "openCode" as const, installed: true },
    { client: "codexCli" as const, installed: true },
  ]);

  expect(clients.map(({ client }) => client)).toEqual([
    "codexCli",
    "openCode",
    "chatgpt",
  ]);
});

test("已安装的智能体按 ChatGPT、Codex CLI、OpenCode 排序", () => {
  const clients = sortAgentClients([
    { client: "openCode" as const, installed: true },
    { client: "codexCli" as const, installed: true },
    { client: "chatgpt" as const, installed: true },
  ]);

  expect(clients.map(({ client }) => client)).toEqual([
    "chatgpt",
    "codexCli",
    "openCode",
  ]);
});
