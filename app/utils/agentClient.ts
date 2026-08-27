import type { AgentClient } from "~/stores/relay";
import codexIcon from "@lobehub/icons-static-svg/icons/codex.svg";
import openaiIcon from "@lobehub/icons-static-svg/icons/openai.svg";
import claudeIcon from "@lobehub/icons-static-svg/icons/claudecode-color.svg";

export const agentClientDefinitions: Array<{
  client: AgentClient;
  label: string;
  icon: string;
  configurable: boolean;
}> = [
  { client: "codexCli", label: "Codex CLI", icon: codexIcon, configurable: true },
  { client: "chatgpt", label: "ChatGPT", icon: openaiIcon, configurable: true },
  { client: "claudeCode", label: "Claude Code", icon: claudeIcon, configurable: true },
];

export function clientSupportsSettings(client: AgentClient) {
  return agentClientDefinitions.some(
    (definition) => definition.client === client && definition.configurable,
  );
}
