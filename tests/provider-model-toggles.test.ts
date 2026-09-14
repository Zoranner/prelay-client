import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import { endpointModelsForProvider } from "../app/utils/endpointModels";
import { setModelCatalog } from "../app/utils/modelCatalog";

const providerSource = [
  "../app/components/providers/ProviderForm.vue",
  "../app/composables/useProviderForm.ts",
]
  .map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
  .join("\n");
const togglesSource = readFileSync(
  new URL(
    "../app/components/providers/ProviderModelToggles.vue",
    import.meta.url,
  ),
  "utf8",
);
const pageSource = readFileSync(
  new URL("../app/pages/providers.vue", import.meta.url),
  "utf8",
);

test("供应商模型清单使用目录显示名但保留模型 ID", () => {
  expect(providerSource).toContain("providerModelOptions");
  expect(togglesSource).toContain("option.label");
  expect(togglesSource).toContain("option.value");
  expect(providerSource).not.toMatch(/v-for="model in languageModels"/);
  expect(providerSource).not.toMatch(/v-for="model in imageGenerationModels"/);
});

test("供应商模型可以点击启停并随表单提交禁用清单", () => {
  expect(providerSource).toContain(':disabled-models="disabledModels"');
  expect(providerSource).toContain('@toggle="toggleModelEnabled"');
  expect(providerSource).toContain(
    "disabled_models: [...disabledModels.value]",
  );
  expect(pageSource).toContain("disabled_models: payload.disabled_models");
});

test("禁用状态通过独立组件呈现，不改动 Tag 用途", () => {
  expect(togglesSource).toContain('type="button"');
  expect(togglesSource).toContain(":aria-pressed=");
  expect(togglesSource).toContain("model-toggle--off");
  expect(togglesSource).not.toContain("@stellar/ui");
});

test("接入点候选排除供应商已禁用的模型", () => {
  setModelCatalog({
    language_models: [
      { id: "provider-model-a", display_name: "Provider Model A" } as never,
      { id: "provider-model-b", display_name: "Provider Model B" } as never,
    ],
    image_generation_models: [],
    providers: [
      {
        id: "toggled-provider",
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
    provider_type: "toggled-provider",
    base_url: "https://example.test",
    api_key: "",
    api_key_masked: "********",
    capabilities: {},
    upstream_protocols: ["openai"],
    disabled_models: ["provider-model-b"],
    created_at: "2026-09-14T00:00:00Z",
  });

  expect(models.map((model) => model.model_name)).toEqual(["provider-model-a"]);
});
