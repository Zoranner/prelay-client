import { expect, test } from "bun:test";
import { groupEndpointModels } from "../app/utils/endpointModels";

test("接入点按对外模型归组并保留全部供应商映射", () => {
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
