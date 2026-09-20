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
const providerFormSource = readFileSync(
  new URL("../app/components/providers/ProviderForm.vue", import.meta.url),
  "utf8",
);

test("供应商模型清单使用目录显示名但保留模型 ID", () => {
  expect(providerSource).toContain("providerModelOptions");
  expect(togglesSource).toContain("model.label");
  expect(providerSource).not.toMatch(/v-for="model in languageModels"/);
  expect(providerSource).not.toMatch(/v-for="model in imageGenerationModels"/);
});

test("供应商表单显式导入模型开关组件", () => {
  // 目录组件名带路径前缀，缺省解析不到这个标签，模板会静默渲染为空。
  expect(providerFormSource).toContain("import ProviderModelToggles");
  expect(providerFormSource).toContain(
    'from "~/components/providers/ProviderModelToggles.vue"',
  );
});

test("供应商模型点击启停并随表单提交启用清单", () => {
  expect(providerSource).toContain(':models="languageToggleModels"');
  expect(providerSource).toContain('@toggle="toggleModel"');
  expect(providerSource).toContain("models: [...enabledModels.value]");
  expect(pageSource).toContain("models: payload.models");
});

test("模型开关用组件库的标签呈现，不用文字标注启停", () => {
  expect(togglesSource).toContain('from "@stellar/ui"');
  expect(togglesSource).toContain("<Tag");
  expect(togglesSource).toContain(":aria-pressed=");
  expect(togglesSource).toContain('emit("remove"');
  expect(togglesSource).toContain("ph:warning-circle");
  expect(togglesSource).toContain('semantic: "error" as const');
  expect(togglesSource).not.toContain("已启用");
  expect(togglesSource).not.toContain("未启用");
});

test("模型清单的提示与空态用组件库组件呈现", () => {
  expect(providerFormSource).toContain("<Alert");
  expect(providerFormSource).toContain("<EmptyState");
  expect(providerFormSource).not.toContain("retired-hint");
  expect(providerFormSource).not.toContain("empty-text");
});

test("已下架模型必须移除后才能保存", () => {
  expect(providerSource).toContain("retiredModels");
  expect(providerSource).toContain("请先移除目录里已下架的模型");
  expect(providerFormSource).toContain("retiredToggleModels");
  expect(providerFormSource).toContain("<span>已下架</span>");
  expect(providerFormSource).not.toContain("目录里查不到");
  expect(providerFormSource).toContain(
    "该供应商已下架：现有配置可在短期内继续使用",
  );
  expect(providerSource).toContain("该供应商的目录条目已下架，只能删除。");
  expect(pageSource).toContain("editingRetired");
});

test("接入点候选来自供应商自己的模型清单", () => {
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
    models: ["provider-model-b"],
    created_at: "2026-09-14T00:00:00Z",
  });

  expect(models.map((model) => model.model_name)).toEqual(["provider-model-b"]);
  expect(models.map((model) => model.display_name)).toEqual([
    "Provider Model B",
  ]);
});
