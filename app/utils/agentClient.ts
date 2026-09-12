import type { AgentClient, AgentItemKind } from "~/stores/relay";
import codexIcon from "@lobehub/icons-static-svg/icons/codex.svg";
import openaiIcon from "@lobehub/icons-static-svg/icons/openai.svg";
import openCodeIcon from "@lobehub/icons-static-svg/icons/opencode.svg";
import claudeCodeIcon from "@lobehub/icons-static-svg/icons/claudecode.svg";

export type AgentSection = "rules" | AgentItemKind;

export const agentSectionOptions: Array<{
  value: AgentSection;
  label: string;
  icon: string;
}> = [
  { value: "rules", label: "规则", icon: "ph:notebook" },
  { value: "mcp", label: "MCP", icon: "ph:terminal-window" },
  { value: "skill", label: "Skill", icon: "ph:book-open-text" },
];

export const agentClientDefinitions: Array<{
  client: AgentClient;
  label: string;
  icon: string;
  configurable: boolean;
  monochrome: boolean;
  sections: AgentSection[];
}> = [
  {
    client: "codexCli",
    label: "Codex CLI",
    icon: codexIcon,
    configurable: true,
    monochrome: true,
    sections: ["rules", "mcp", "skill"],
  },
  {
    client: "chatgpt",
    label: "ChatGPT",
    icon: openaiIcon,
    configurable: true,
    monochrome: true,
    sections: ["rules", "mcp", "skill"],
  },
  {
    client: "openCode",
    label: "OpenCode",
    icon: openCodeIcon,
    configurable: true,
    monochrome: true,
    sections: ["rules", "mcp", "skill"],
  },
  {
    client: "claudeCode",
    label: "Claude Code",
    icon: claudeCodeIcon,
    configurable: true,
    monochrome: true,
    sections: ["rules", "mcp", "skill"],
  },
];

export const agentClients = agentClientDefinitions.map(({ client }) => client);

const agentClientPriority: Record<AgentClient, number> = {
  chatgpt: 0,
  codexCli: 1,
  claudeCode: 2,
  openCode: 3,
};

export function sortAgentClients<
  T extends { client: AgentClient; installed: boolean },
>(clients: T[]) {
  return [...clients].sort(
    (left, right) =>
      Number(right.installed) - Number(left.installed) ||
      agentClientPriority[left.client] - agentClientPriority[right.client],
  );
}

export function clientSupportsSettings(client: AgentClient) {
  return agentClientDefinitions.some(
    (definition) => definition.client === client && definition.configurable,
  );
}

export function clientSupportsRules(client: AgentClient) {
  return agentClientDefinitions.some(
    (definition) =>
      definition.client === client && definition.sections.includes("rules"),
  );
}
