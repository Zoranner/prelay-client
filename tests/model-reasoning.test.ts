import { describe, expect, test } from "bun:test";
import type { CatalogLanguageModelResponse } from "~/stores/relay";
import {
  normalizeReasoningEffort,
  reasoningEffortOptions,
} from "~/utils/modelReasoning";

function model(
  reasoning_efforts: string[] | null,
  default_reasoning_effort: string | null = null,
): CatalogLanguageModelResponse {
  return {
    id: "test-model",
    display_name: "Test model",
    description: null,
    reasoning_efforts,
    default_reasoning_effort,
    context_window: null,
    max_context_window: null,
    effective_context_window_percent: null,
    input_modalities: null,
    supports_parallel_tool_calls: null,
    supports_reasoning_summaries: null,
    supports_image_detail_original: null,
    support_verbosity: null,
    default_verbosity: null,
    apply_patch_tool_type: null,
    web_search_tool_type: null,
    truncation_policy: null,
    reasoning_summary_format: null,
    default_reasoning_summary: null,
    shell_type: null,
    visibility: null,
    supported_in_api: null,
    priority: null,
    base_instructions: null,
    experimental_supported_tools: null,
    minimal_client_version: null,
  };
}

describe("reasoning effort adapter", () => {
  test("keeps all six catalog options in server order", () => {
    expect(
      reasoningEffortOptions(
        model(["none", "minimal", "low", "medium", "high", "max"], "high"),
        false,
      ),
    ).toEqual([
      { value: "", label: "跟随模型默认（high）" },
      { value: "none", label: "无" },
      { value: "minimal", label: "极低" },
      { value: "low", label: "低" },
      { value: "medium", label: "中" },
      { value: "high", label: "高" },
      { value: "max", label: "最高" },
    ]);
  });

  test("keeps a three-level catalog model in its declared order", () => {
    expect(
      reasoningEffortOptions(model(["low", "medium", "high"], "high"), false),
    ).toEqual([
      { value: "", label: "跟随模型默认（high）" },
      { value: "low", label: "低" },
      { value: "medium", label: "中" },
      { value: "high", label: "高" },
    ]);
  });

  test("returns no options for a model without reasoning support", () => {
    expect(reasoningEffortOptions(model([]), false)).toEqual([]);
    expect(reasoningEffortOptions(undefined, false)).toEqual([]);
  });

  test("uses the established generic options for custom connections", () => {
    expect(reasoningEffortOptions(model(["none", "max"], "max"), true)).toEqual([
      { value: "low", label: "低" },
      { value: "medium", label: "中" },
      { value: "high", label: "高" },
      { value: "xhigh", label: "很高" },
    ]);
  });

  test("preserves supported values and clears unsupported overrides", () => {
    const catalogModel = model(["minimal", "high", "max"], "high");

    expect(normalizeReasoningEffort("max", catalogModel)).toBe("max");
    expect(normalizeReasoningEffort("medium", catalogModel)).toBe("");
    expect(normalizeReasoningEffort("", catalogModel)).toBe("");
    expect(normalizeReasoningEffort("high", undefined)).toBe("");
  });
});
