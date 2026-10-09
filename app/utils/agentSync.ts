import type {
  AgentClient,
  CodexSettings,
  OpenCodeSettings,
  RelayEndpoint,
} from "~/stores/relay";
import {
  endpointCatalogModels,
  managementBaseUrl,
  normalizeBaseUrl,
} from "~/utils/agentConnections";

export type AgentEndpointLinks = {
  codex: string | null;
  opencode: string | null;
};

export function agentHost(client: AgentClient) {
  return client === "openCode" ? ("opencode" as const) : ("codex" as const);
}

/**
 * 找出这个宿主当前引用的接入点。
 *
 * 优先用本地记录（改名、重置 Token 也认得出）；没有记录时按现有依据认领一次：
 * Codex/ChatGPT 比接入点名，OpenCode 比 token。
 */
export function linkedEndpoint(options: {
  client: AgentClient;
  settings: CodexSettings | OpenCodeSettings;
  links: AgentEndpointLinks;
  endpoints: RelayEndpoint[];
  relayUrl: string;
}) {
  const host = agentHost(options.client);
  const recorded = options.links[host];
  if (recorded) {
    return options.endpoints.find((endpoint) => endpoint.id === recorded);
  }
  const baseUrl =
    (options.settings as { baseUrl?: string }).baseUrl?.trim() ?? "";
  if (
    !baseUrl ||
    normalizeBaseUrl(baseUrl) !== managementBaseUrl(options.relayUrl)
  ) {
    return undefined;
  }
  return options.client === "openCode"
    ? options.endpoints.find(
        (endpoint) =>
          endpoint.token ===
          (options.settings as OpenCodeSettings).endpointToken,
      )
    : options.endpoints.find(
        (endpoint) =>
          endpoint.name === (options.settings as CodexSettings).endpointName,
      );
}

/**
 * 默认模型在接入点里已经不存在时回退到剩余模型的第一个。
 *
 * 接入点没有可用模型时返回 null，调用方跳过这次对账，不写半套配置。
 */
export function agentModelFallback(endpoint: RelayEndpoint, selected?: string) {
  const models = endpointCatalogModels(endpoint);
  const current = selected?.trim();
  if (current && models.some((model) => model.id === current)) {
    return { model: current };
  }
  const next = models[0];
  if (!next) return null;
  return current
    ? { model: next.id, fallback: { from: current, to: next.id } }
    : { model: next.id };
}
