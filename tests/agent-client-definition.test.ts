import { expect, test } from "bun:test";

import { agentClientDefinitions } from "../app/utils/agentClient";

test("智能体客户端按独立安装形态保留产品与图标身份", () => {
  expect(agentClientDefinitions).toEqual([
    {
      client: "codexCli",
      label: "Codex CLI",
      icon: expect.stringContaining("codex.svg"),
      configurable: true,
    },
    {
      client: "chatgpt",
      label: "ChatGPT",
      icon: expect.stringContaining("openai.svg"),
      configurable: true,
    },
    {
      client: "claudeCode",
      label: "Claude Code",
      icon: expect.stringContaining("claudecode-color.svg"),
      configurable: true,
    },
  ]);
});
