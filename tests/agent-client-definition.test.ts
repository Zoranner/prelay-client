import { expect, test } from "bun:test";

import { agentClientDefinitions } from "../app/utils/agentClient";

test("智能体客户端按独立安装形态保留产品与图标身份", () => {
  expect(agentClientDefinitions).toEqual([
    {
      client: "codexCli",
      label: "Codex CLI",
      icon: expect.stringContaining("codex.svg"),
      configurable: true,
      monochrome: true,
      sections: ["rules", "mcp", "skill"],
    },
    {
      client: "chatgpt",
      label: "ChatGPT",
      icon: expect.stringContaining("openai.svg"),
      configurable: true,
      monochrome: true,
      sections: ["rules", "mcp", "skill"],
    },
    {
      client: "openCode",
      label: "OpenCode",
      icon: expect.stringContaining("opencode.svg"),
      configurable: true,
      monochrome: true,
      sections: ["rules", "mcp", "skill"],
    },
  ]);
});
