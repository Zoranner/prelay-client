import { expect, test } from "bun:test";

import {
  agentModelFallback,
  linkedEndpoint,
  type AgentEndpointLinks,
} from "../app/utils/agentSync";
import { setModelCatalog } from "../app/utils/modelCatalog";

const RELAY_URL = "https://relay.example.test";

function endpoint(id: string, name: string, token: string, modelIds: string[]) {
  return {
    id,
    name,
    protocol: "all",
    token,
    created_at: "2026-10-09T00:00:00Z",
    models: modelIds.map((modelId, index) => ({
      id: `${id}-${modelId}`,
      endpoint_id: id,
      model_name: modelId,
      display_name: `${modelId} 显示名`,
      provider_id: "provider-a",
      upstream_model: modelId,
      created_at: "2026-10-09T00:00:00Z",
      candidate_order: index,
    })),
  } as never;
}

function useCatalog(modelIds: string[]) {
  setModelCatalog({
    language_models: modelIds.map((id) => ({
      id,
      display_name: `${id} 显示名`,
      reasoning_efforts: ["low", "high"],
    })) as never,
    image_generation_models: [],
    providers: [],
  });
}

const noLinks: AgentEndpointLinks = { codex: null, opencode: null };

test("按本地记录认出宿主引用的接入点", () => {
  useCatalog(["k3", "k3-mini"]);
  const endpoints = [
    endpoint("endpoint-1", "旧名字", "token-1", ["k3"]),
    endpoint("endpoint-2", "Endpoint 2", "token-2", ["k3-mini"]),
  ];

  const resolved = linkedEndpoint({
    client: "codexCli",
    settings: { endpointName: "旧名字", baseUrl: `${RELAY_URL}/v1` },
    links: { codex: "endpoint-2", opencode: null },
    endpoints,
    relayUrl: RELAY_URL,
  });

  // 记录优先：接入点改名后仍认得出，也压过按名字反推的结果。
  expect(resolved?.id).toBe("endpoint-2");
});

test("没有记录时按名字或 token 认领一次", () => {
  useCatalog(["k3"]);
  const endpoints = [endpoint("endpoint-1", "Endpoint 1", "token-1", ["k3"])];

  expect(
    linkedEndpoint({
      client: "codexCli",
      settings: { endpointName: "Endpoint 1", baseUrl: `${RELAY_URL}/v1` },
      links: noLinks,
      endpoints,
      relayUrl: RELAY_URL,
    })?.id,
  ).toBe("endpoint-1");
  expect(
    linkedEndpoint({
      client: "openCode",
      settings: { endpointToken: "token-1", baseUrl: `${RELAY_URL}/v1` },
      links: noLinks,
      endpoints,
      relayUrl: RELAY_URL,
    })?.id,
  ).toBe("endpoint-1");
  // 服务地址已经换掉的旧配置不参与认领。
  expect(
    linkedEndpoint({
      client: "openCode",
      settings: {
        endpointToken: "token-1",
        baseUrl: "https://old.example.test/v1",
      },
      links: noLinks,
      endpoints,
      relayUrl: RELAY_URL,
    }),
  ).toBeUndefined();
});

test("记录指向已删除的接入点时不下结论", () => {
  useCatalog(["k3"]);
  const endpoints = [endpoint("endpoint-2", "Endpoint 2", "token-2", ["k3"])];

  expect(
    linkedEndpoint({
      client: "codexCli",
      settings: { endpointName: "Endpoint 1", baseUrl: `${RELAY_URL}/v1` },
      links: { codex: "endpoint-1", opencode: null },
      endpoints,
      relayUrl: RELAY_URL,
    }),
  ).toBeUndefined();
});

test("默认模型被删后回退到接入点剩余模型的第一个", () => {
  useCatalog(["k3", "k3-mini"]);
  const single = endpoint("endpoint-1", "Endpoint 1", "token-1", ["k3"]);
  const multi = endpoint("endpoint-1", "Endpoint 1", "token-1", [
    "k3",
    "k3-mini",
  ]);

  expect(agentModelFallback(multi, "k3")).toEqual({ model: "k3" });
  expect(agentModelFallback(multi, "removed-model")).toEqual({
    model: "k3",
    fallback: { from: "removed-model", to: "k3" },
  });
  expect(agentModelFallback(multi, "")).toEqual({ model: "k3" });
  // 接入点没有可用模型时不写配置。
  expect(
    agentModelFallback(endpoint("endpoint-2", "空接入点", "token-2", []), "k3"),
  ).toBeNull();
  expect(agentModelFallback(single, "k3")).toEqual({ model: "k3" });
});
