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
      sections: ["rules", "plugin", "mcp", "skill", "hook"],
    },
    {
      client: "chatgpt",
      label: "ChatGPT",
      icon: expect.stringContaining("openai.svg"),
      configurable: true,
      monochrome: true,
      sections: ["rules", "plugin", "mcp", "skill"],
    },
    {
      client: "openCode",
      label: "OpenCode",
      icon: expect.stringContaining("opencode.svg"),
      configurable: true,
      monochrome: true,
      sections: ["rules", "plugin", "mcp", "skill"],
    },
  ]);
});

test("智能体内部按规则、插件、MCP、Skill、Hook 分类顺序与图标展示", () => {
  expect(agentSectionOptions).toEqual([
    { value: "rules", label: "规则", icon: "ph:notebook" },
    { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
    { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
    { value: "skill", label: "Skill", icon: "ph:book-open-text" },
    { value: "hook", label: "Hook", icon: "ph:webhooks-logo" },
  ]);
});
