import { expect, test } from "bun:test";
import type { CatalogLanguageModelResponse } from "../app/stores/relay";
import {
  normalizeReasoningEffort,
  reasoningEffortOptions,
} from "../app/utils/modelReasoning";
import { saveWithAgentValidation } from "../app/composables/useAgentSettings";
import { setModelCatalog } from "../app/utils/modelCatalog";

const model = (
  id: string,
  reasoning_efforts: string[] | null,
): CatalogLanguageModelResponse =>
  ({ id, display_name: id, reasoning_efforts }) as CatalogLanguageModelResponse;

test("切换目录模型时保留支持的覆盖并清除不支持的覆盖", () => {
  const previous = model("first", ["low", "high"]);
  const next = model("second", ["medium", "high"]);

  expect(normalizeReasoningEffort("high", next)).toBe("high");
  expect(normalizeReasoningEffort("low", next)).toBe("");
  expect(normalizeReasoningEffort("low", previous)).toBe("low");
});

test("无推理档位时禁用控制，目录默认值使用空覆盖表示", () => {
  expect(reasoningEffortOptions(model("none", []), false)).toEqual([]);
  expect(
    reasoningEffortOptions(model("default", ["medium"]), false)[0],
  ).toEqual({
    value: "",
    label: "跟随模型默认（未指定）",
  });
});

test("Prelay 保存拒绝当前模型不支持的推理覆盖且不调用保存", async () => {
  setModelCatalog({
    language_models: [model("chat-model", ["low", "high"])],
    image_generation_models: [],
    providers: [],
  });
  let saves = 0;
  const result = await saveWithAgentValidation({
    kind: "prelay",
    status: "ready",
    selectedModel: "chat-model",
    endpointModelIds: ["chat-model"],
    reasoningEffort: "max",
    save: async () => {
      saves += 1;
    },
  });

  expect(result).toContain("推理强度");
  expect(saves).toBe(0);

  const spacedResult = await saveWithAgentValidation({
    kind: "prelay",
    status: "ready",
    selectedModel: "chat-model",
    endpointModelIds: ["chat-model"],
    reasoningEffort: " high ",
    save: async () => {
      saves += 1;
    },
  });

  expect(spacedResult).toContain("推理强度");
  expect(saves).toBe(0);
});
