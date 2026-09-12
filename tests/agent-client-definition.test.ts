import { expect, test } from "bun:test";

import {
  agentClientDefinitions,
  agentSectionOptions,
} from "../app/utils/agentClient";

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

test("智能体内部和扩展库共用规则、MCP、Skill 分类顺序与图标", () => {
  expect(agentSectionOptions).toEqual([
    { value: "rules", label: "规则", icon: "ph:notebook" },
    { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
    { value: "skill", label: "Skill", icon: "ph:book-open-text" },
  ]);
});
