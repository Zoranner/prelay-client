import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import {
  loadModelCatalogRequest,
  modelCatalogEntry,
  modelCatalogProviderModels,
  setModelCatalogStatus,
  setModelCatalog,
  useModelCatalog,
} from "../app/utils/modelCatalog";
import type { ProviderCatalogResponse } from "../app/stores/relay";
import { groupEndpointModels } from "../app/utils/endpointModels";
import {
  saveWithAgentValidation,
  validatePrelayModelSelection,
} from "../app/composables/useAgentSettings";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("模型目录暴露显式加载状态并保留完整索引", () => {
  const catalog = useModelCatalog();
  setModelCatalog(undefined);
  expect(catalog.status.value).toBe("idle");
  expect(catalog.catalog.value.language_models).toHaveLength(0);

  const languageModel = {
    id: "chat-model",
    display_name: "Chat Model",
    description: null,
    reasoning_efforts: null,
    default_reasoning_effort: null,
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
    base_instructions: "instructions",
    experimental_supported_tools: null,
    minimal_client_version: null,
  };
  setModelCatalog({
    language_models: [languageModel],
    image_generation_models: [],
    providers: [
      {
        id: "provider-a",
        name: "Provider A",
        auth_scheme: "bearer",
        base_url: "https://example.test",
        protocols: ["chat_completions"],
        protocol_base_urls: [],
        language_models: ["chat-model"],
        image_generation_models: [],
      },
    ],
  });
  expect(catalog.status.value).toBe("ready");
  expect(modelCatalogEntry("chat-model")).toEqual(languageModel);
  expect(modelCatalogProviderModels("provider-a")).toEqual([languageModel]);
});

test("目录请求状态和 URL 请求代次必须防止旧响应覆盖", () => {
  const app = source("app.vue");
  const catalog = source("utils/modelCatalog.ts");
  expect(catalog).toContain('setModelCatalogStatus("loading")');
  expect(catalog).toContain("setModelCatalog(value)");
  expect(catalog).toContain('setModelCatalogStatus("error")');
  expect(app).toContain("modelCatalogRequestId");
  expect(app).toContain("modelCatalogUrl = null");
  expect(app).toMatch(/requestId === modelCatalogRequestId/);
});

test("实际目录请求协调器报告 loading、error 并丢弃过期响应", async () => {
  const catalog = useModelCatalog();
  const value = (id: string) =>
    ({
      language_models: [{ id }],
      image_generation_models: [],
      providers: [],
    }) as ProviderCatalogResponse;
  let current = true;
  let resolveRequest!: (result: ProviderCatalogResponse) => void;
  const pending = new Promise<ProviderCatalogResponse>((resolve) => {
    resolveRequest = resolve;
  });
  const request = loadModelCatalogRequest(
    () => pending,
    () => current,
  );
  expect(catalog.status.value).toBe("loading");
  current = false;
  resolveRequest(value("stale"));
  await request;
  expect(catalog.status.value).toBe("loading");
  expect(modelCatalogEntry("stale")).toBeUndefined();

  current = true;
  await loadModelCatalogRequest(
    async () => {
      throw new Error("offline");
    },
    () => current,
  );
  expect(catalog.status.value).toBe("error");
});

test("目录不可用时阻止 Prelay 智能体保存但不阻止自定义连接", () => {
  const settings = source("composables/useAgentSettings.ts");
  expect(settings).toContain("validatePrelayModelSelection");
  expect(settings).toContain("目录尚未加载完成");
  expect(settings).toContain("modelCatalogEntry");
  expect(settings).toContain("catalogLanguageModel");
  expect(settings).toContain("return null");
  expect(settings).toContain('kind: "custom"');
});

test("Prelay 模型校验覆盖目录状态、接入点归属和语言模型类型", () => {
  setModelCatalog(undefined);
  expect(
    validatePrelayModelSelection("idle", "chat-model", ["chat-model"]),
  ).toContain("尚未加载");
  setModelCatalog({
    language_models: [],
    image_generation_models: [
      {
        id: "image-model",
        display_name: "Image Model",
        description: null,
        input_modalities: null,
        output_modalities: null,
        sizes: null,
        quality_options: null,
        background_options: null,
        output_formats: null,
        supports_editing: null,
        supports_mask: null,
        supports_reference_images: null,
        visibility: null,
        supported_in_api: null,
        priority: null,
      },
    ],
    providers: [],
  });
  expect(
    validatePrelayModelSelection("ready", "image-model", ["image-model"]),
  ).toContain("目录外模型");
  expect(
    groupEndpointModels([
      {
        model_name: "legacy-public-id",
        provider_id: "provider-a",
        upstream_model: "chat-model",
      },
    ])[0]?.name,
  ).toBe("legacy-public-id");
});

test("实际智能体保存门禁阻断 Prelay、放行 Custom 并在 ready 时只保存一次", async () => {
  let saves = 0;
  const save = async () => {
    saves += 1;
  };
  setModelCatalogStatus("error");
  expect(
    await saveWithAgentValidation({
      kind: "prelay",
      status: "error",
      selectedModel: "chat-model",
      endpointModelIds: ["chat-model"],
      save,
    }),
  ).toContain("尚未加载");
  expect(saves).toBe(0);

  expect(
    await saveWithAgentValidation({
      kind: "custom",
      status: "error",
      selectedModel: "custom-model",
      endpointModelIds: [],
      save,
    }),
  ).toBeNull();
  expect(saves).toBe(1);

  setModelCatalog({
    language_models: [{ id: "chat-model", reasoning_efforts: [] } as never],
    image_generation_models: [],
    providers: [],
  });
  expect(
    await saveWithAgentValidation({
      kind: "prelay",
      status: "ready",
      selectedModel: "chat-model",
      endpointModelIds: ["chat-model"],
      save,
    }),
  ).toBeNull();
  expect(saves).toBe(2);
});

test("Provider 与 Endpoint 模型选项只能来自完整目录", () => {
  const provider = source("composables/useProviderForm.ts");
  const endpoint = source("components/endpoints/EndpointForm.vue");
  const endpointModels = source("utils/endpointModels.ts");
  expect(provider).toContain("modelCatalogProviderModels");
  expect(provider).not.toContain("providers_discover_models");
  expect(endpointModels).toContain("modelCatalogProviderModels");
  expect(endpoint).not.toContain('label="对外模型名"');
});
