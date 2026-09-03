import { expect, test } from "bun:test";
import { groupEndpointModels } from "../app/utils/endpointModels";
import { modelCatalogEntry, setModelCatalog } from "../app/utils/modelCatalog";

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
