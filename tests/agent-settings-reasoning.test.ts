import { expect, test } from "bun:test";
import type { CatalogLanguageModelResponse } from "../app/stores/relay";
import {
  normalizeReasoningEffort,
  reasoningEffortOptions,
} from "../app/utils/modelReasoning";
import {
  claudeCodeBaseUrlMatchesRelay,
  saveWithAgentValidation,
} from "../app/composables/useAgentSettings";
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

test("无推理档位时禁用控制，目录选项不包含跟随默认", () => {
  expect(reasoningEffortOptions(model("none", []), false)).toEqual([]);
  expect(
    reasoningEffortOptions(model("default", ["medium"]), false)[0],
  ).toEqual({
    value: "medium",
    label: "中",
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

test("Prelay 保存将纯空白推理覆盖视为目录默认且调用保存", async () => {
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
    reasoningEffort: "  \t ",
    save: async () => {
      saves += 1;
    },
  });

  expect(result).toBeNull();
  expect(saves).toBe(1);
});

test("Claude Code 保存后的服务根地址可以匹配当前服务地址", () => {
  expect(
    claudeCodeBaseUrlMatchesRelay(
      "https://relay.example.test",
      " https://relay.example.test/ ",
    ),
  ).toBe(true);
});
