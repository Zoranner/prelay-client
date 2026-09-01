import type { AgentClient, AgentItemKind } from "~/stores/relay";
import codexIcon from "@lobehub/icons-static-svg/icons/codex.svg";
import openaiIcon from "@lobehub/icons-static-svg/icons/openai.svg";
import openCodeIcon from "@lobehub/icons-static-svg/icons/opencode.svg";

export const agentClientDefinitions: Array<{
  client: AgentClient;
  label: string;
  icon: string;
  configurable: boolean;
  monochrome: boolean;
  sections: Array<"rules" | AgentItemKind>;
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
];

export const agentClients = agentClientDefinitions.map(({ client }) => client);

const agentClientPriority: Record<AgentClient, number> = {
  chatgpt: 0,
  codexCli: 1,
  openCode: 2,
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
