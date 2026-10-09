import type {
  CatalogLanguageModelResponse,
  CatalogModelResponse,
  RelayEndpoint,
} from "~/stores/relay";
import { groupEndpointModels } from "~/utils/endpointModels";

/** 接入点对应的 Codex 连接参数（ChatGPT 与 Codex CLI 共用一份宿主配置）。 */
export type CodexConnectionDraft = {
  kind: "prelay";
  endpointId: string;
  endpointName: string;
  endpointToken: string;
  relayUrl: string;
  models: CatalogLanguageModelResponse[];
};

export type OpenCodeConnectionDraft = {
  kind: "prelay";
  endpointId: string;
  relayUrl: string;
  endpointToken: string;
  models: CatalogLanguageModelResponse[];
};

export function normalizeBaseUrl(url: string) {
  return url.trim().replace(/\/+$/, "");
}

/** 管理服务的调用前缀：接入点的 base_url 固定落在 `<服务地址>/v1`。 */
export function managementBaseUrl(relayUrl: string) {
  const normalized = normalizeBaseUrl(relayUrl);
  return normalized.endsWith("/v1") ? normalized : `${normalized}/v1`;
}

export function catalogLanguageModel(
  model: CatalogModelResponse | undefined,
): CatalogLanguageModelResponse | undefined {
  return model && "reasoning_efforts" in model ? model : undefined;
}

/** 接入点的模型清单：按对外模型 id 归组后映射到目录里的语言模型。 */
export function endpointCatalogModels(endpoint: RelayEndpoint) {
  return groupEndpointModels(endpoint.models)
    .map((group) => catalogLanguageModel(group.catalogModel))
    .filter((model): model is CatalogLanguageModelResponse => Boolean(model));
}

export function codexConnectionFor(
  endpoint: RelayEndpoint,
  relayUrl: string,
): CodexConnectionDraft {
  return {
    kind: "prelay",
    endpointId: endpoint.id,
    endpointName: endpoint.name,
    endpointToken: endpoint.token,
    relayUrl,
    models: endpointCatalogModels(endpoint),
  };
}

export function openCodeConnectionFor(
  endpoint: RelayEndpoint,
  relayUrl: string,
): OpenCodeConnectionDraft {
  return {
    kind: "prelay",
    endpointId: endpoint.id,
    relayUrl,
    endpointToken: endpoint.token,
    models: endpointCatalogModels(endpoint),
  };
}
