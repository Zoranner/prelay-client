import { expect, test } from "bun:test";
import {
  availableEndpointModelsForProvider,
  checkEndpointMapping,
  endpointMappingIssue,
  endpointMappingProblems,
  endpointModelsForProvider,
  groupEndpointModels,
  moveEndpointMapping,
  normalizeEndpointModelIdentities,
} from "../app/utils/endpointModels";
import { modelCatalogEntry, setModelCatalog } from "../app/utils/modelCatalog";
import { readFileSync } from "node:fs";

test("接入点候选顺序只在同一模型分组内上下调整", () => {
  const models = [
    { provider_id: "a", upstream_model: "luna", model_name: "luna" },
    { provider_id: "b", upstream_model: "terra", model_name: "terra" },
    { provider_id: "c", upstream_model: "luna-2026-08", model_name: "luna" },
  ];

  expect(
    moveEndpointMapping(models, 2, -1).map((model) => model.provider_id),
  ).toEqual(["c", "b", "a"]);
  expect(
    moveEndpointMapping(models, 0, 1).map((model) => model.provider_id),
  ).toEqual(["c", "b", "a"]);
  expect(
    moveEndpointMapping(models, 0, -1).map((model) => model.provider_id),
  ).toEqual(["a", "b", "c"]);
  expect(
    moveEndpointMapping(models, 2, 1).map((model) => model.provider_id),
  ).toEqual(["a", "b", "c"]);
  expect(
    moveEndpointMapping(models, 1, 1).map((model) => model.provider_id),
  ).toEqual(["a", "b", "c"]);
});

test("接入点按对外模型 ID 归组并保留全部供应商路由", () => {
  const catalogModel = {
    id: "gpt-5.6-luna",
    display_name: "GPT-5.6 Luna",
    description: "测试目录模型",
  };
  setModelCatalog({
    language_models: [catalogModel],
    image_generation_models: [],
    providers: [],
  });
  expect(modelCatalogEntry("gpt-5.6-luna")).toEqual(catalogModel);
  const models = [
    {
      provider_id: "provider-a",
      upstream_model: "gpt-5.6-luna",
      model_name: "gpt-5.6-luna",
    },
    {
      provider_id: "provider-b",
      upstream_model: "gpt-5.6-luna-2026-08",
      model_name: "gpt-5.6-luna",
    },
    {
      provider_id: "provider-a",
      upstream_model: "gpt-5.6-terra",
      model_name: "gpt-5.6-terra",
    },
  ];

  const groups = groupEndpointModels(models);

  expect(groups.map((group) => group.name)).toEqual([
    "gpt-5.6-luna",
    "gpt-5.6-terra",
  ]);
  expect(groups[0]?.displayName).toBe("GPT-5.6 Luna");
  expect(groups[0]?.catalogModel).toEqual(catalogModel);
  expect(
    groups[0]?.mappings.map((mapping) => mapping.model.provider_id),
  ).toEqual(["provider-a", "provider-b"]);
  expect(groups[0]?.mappings.map((mapping) => mapping.index)).toEqual([0, 1]);
});

test("编辑接入点回程把上游名还原为目录模型 id", () => {
  const catalogModelsForProvider = (providerType: string) =>
    providerType === "tokenharbor"
      ? [{ id: "deepseek-flash", display_name: "DeepSeek V4.1 Flash" }]
      : [];
  const providerTypeFor = (providerId: string) => {
    const resolved =
      providerId === "provider-tokenharbor" ? "tokenharbor" : undefined;
    return resolved;
  };
  const storedMappings = [
    {
      provider_id: "provider-tokenharbor",
      upstream_model: "deepseek-v4.1-flash",
      model_name: "deepseek-flash",
    },
  ];
  const [mapping] = normalizeEndpointModelIdentities(
    storedMappings,
    providerTypeFor,
    catalogModelsForProvider,
  );

  expect(mapping.upstream_model).toBe("deepseek-flash");
  expect(
    normalizeEndpointModelIdentities(
      [mapping],
      providerTypeFor,
      catalogModelsForProvider,
    ),
  ).toEqual([mapping]);
});

test("接入点新增候选使用供应商自己的模型清单", () => {
  setModelCatalog({
    language_models: [
      {
        id: "provider-model-a",
        display_name: "Provider Model A",
      } as never,
      {
        id: "provider-model-b",
        display_name: "Provider Model B",
      } as never,
    ],
    image_generation_models: [],
    providers: [
      {
        id: "legacy-provider",
        name: "Provider A",
        auth_scheme: "bearer",
        base_url: "https://example.test",
        protocols: ["chat_completions"],
        protocol_base_urls: [],
        language_models: ["provider-model-a", "provider-model-b"],
        image_generation_models: [],
      },
    ],
  });
  const models = endpointModelsForProvider({
    id: "provider-a",
    name: "Provider A",
    provider_type: "legacy-provider",
    base_url: "https://example.test",
    api_key: "",
    api_key_masked: "********",
    capabilities: {},
    upstream_protocols: ["openai"],
    models: ["provider-model-b"],
    created_at: "2026-09-04T00:00:00Z",
  });

  expect(models.map((model) => model.model_name)).toEqual(["provider-model-b"]);
});

test("接入点新增候选排除当前供应商已绑定模型但保留其他供应商候选", () => {
  const providerModels = [
    {
      id: "provider-model-luna",
      provider_id: "provider-a",
      model_name: "gpt-5.6-luna",
      created_at: "2026-09-04T00:00:00Z",
    },
    {
      id: "provider-model-terra",
      provider_id: "provider-a",
      model_name: "gpt-5.6-terra",
      created_at: "2026-09-04T00:00:00Z",
    },
  ];
  const endpointModels = [
    {
      provider_id: "provider-a",
      upstream_model: "gpt-5.6-terra",
      model_name: "gpt-5.6-terra",
    },
  ];

  expect(
    availableEndpointModelsForProvider(
      providerModels,
      endpointModels,
      "provider-a",
    ).map((model) => model.model_name),
  ).toEqual(["gpt-5.6-luna"]);
  expect(
    availableEndpointModelsForProvider(
      providerModels,
      endpointModels,
      "provider-b",
    ).map((model) => model.model_name),
  ).toEqual(["gpt-5.6-luna", "gpt-5.6-terra"]);
});

test("未命名对外模型以其上游模型名作为组名", () => {
  const groups = groupEndpointModels([
    {
      provider_id: "provider-a",
      upstream_model: "gpt-5.6-sol",
      model_name: "   ",
    },
  ]);

  expect(groups.map((group) => group.name)).toEqual(["gpt-5.6-sol"]);
});

test("未知对外模型 ID 的组显示名称回退为 ID", () => {
  setModelCatalog({
    language_models: [],
    image_generation_models: [],
    providers: [],
  });

  const [group] = groupEndpointModels([
    {
      provider_id: "provider-a",
      upstream_model: "upstream-model",
      model_name: "legacy-model-id",
    },
  ]);

  expect(group?.displayName).toBe("legacy-model-id");
  expect(group?.catalogModel).toBeUndefined();
});

test("接入点归组优先使用服务端模型显示名", () => {
  const [group] = groupEndpointModels([
    {
      model_name: "model-id",
      upstream_model: "upstream-id",
      provider_id: "provider-a",
      display_name: "服务端显示名",
    },
  ]);

  expect(group.displayName).toBe("服务端显示名");
});

test("共享 Provider 选项保留原始 ID并标记只读来源", () => {
  const source = readFileSync(
    new URL("../app/utils/endpointModels.ts", import.meta.url),
    "utf8",
  );

  expect(source).toContain("export function providerOptionLabel");
  expect(source).toContain("owner_display_name");
  expect(source).toContain("const readOnly = provider.can_manage === false");
  expect(source).toContain("read_only: readOnly");
  expect(source).toContain("provider.id");
});

test("接入点新增候选按对外模型 ID 排除已绑定模型，忽略供应商上游名差异", () => {
  const providerModels = [
    { model_name: "k3", display_name: "Kimi K3" },
    { model_name: "k3-256k", display_name: "Kimi K3 256K" },
  ];
  const endpointModels = [
    {
      provider_id: "provider-a",
      model_name: "k3",
      upstream_model: "kimi-k3",
    },
  ];

  expect(
    availableEndpointModelsForProvider(
      providerModels,
      endpointModels,
      "provider-a",
    ).map((model) => model.model_name),
  ).toEqual(["k3-256k"]);
});

test("接入点映射校验按对外模型 ID 判定重复", () => {
  const providerModels = [{ model_name: "k3", display_name: "Kimi K3" }];
  const endpointModels = [
    {
      provider_id: "provider-a",
      model_name: "k3",
      upstream_model: "kimi-k3",
    },
  ];

  expect(
    checkEndpointMapping({
      providerId: "provider-a",
      upstreamModel: "k3",
      providerModels,
      endpointModels,
    }),
  ).toEqual({ message: "该供应商已经绑定此模型。", title: "模型已存在" });
  expect(
    checkEndpointMapping({
      providerId: "provider-b",
      upstreamModel: "k3",
      providerModels,
      endpointModels,
    }),
  ).toBeNull();
});

test("接入点旧映射按供应商自己的模型清单判定失效", () => {
  setModelCatalog({
    language_models: [],
    image_generation_models: [],
    providers: [],
  });
  const providers = [{ id: "provider-a", name: "Provider A", models: ["k3"] }];

  expect(
    endpointMappingIssue(
      { provider_id: "provider-a", upstream_model: "k3", model_name: "k3" },
      providers,
    ),
  ).toBeNull();
  // 目录下架导致上游名无法归一化时，存回的模型 ID 仍在清单里就不误报。
  expect(
    endpointMappingIssue(
      {
        provider_id: "provider-a",
        upstream_model: "kimi-k3",
        model_name: "k3",
      },
      providers,
    ),
  ).toBeNull();
  expect(
    endpointMappingIssue(
      {
        provider_id: "provider-a",
        upstream_model: "kimi-k3",
        model_name: "k3-retired",
      },
      providers,
    ),
  ).toBe("model_removed");
  expect(
    endpointMappingIssue(
      { provider_id: "provider-gone", upstream_model: "k3", model_name: "k3" },
      providers,
    ),
  ).toBe("provider_unavailable");
});

test("接入点失效映射只保留模型与原因", () => {
  setModelCatalog({
    language_models: [],
    image_generation_models: [],
    providers: [],
  });
  const providers = [{ id: "provider-a", name: "Provider A", models: ["k3"] }];
  const removed = {
    provider_id: "provider-a",
    upstream_model: "k3-retired",
    model_name: "k3-retired",
    display_name: "Kimi K3 Retired",
  };
  const removedProvider = {
    provider_id: "provider-gone",
    upstream_model: "k3",
    model_name: "k3",
  };
  const healthy = {
    provider_id: "provider-a",
    upstream_model: "k3",
    model_name: "k3",
  };

  expect(
    endpointMappingProblems([removed, removedProvider, healthy], providers),
  ).toEqual([
    { model: removed, issue: "model_removed" },
    { model: removedProvider, issue: "provider_unavailable" },
  ]);
});
