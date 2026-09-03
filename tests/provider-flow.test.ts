import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { setModelCatalog } from "../app/utils/modelCatalog";
import { providerModelOptions } from "../app/utils/providerTemplates";

const providerSource = [
  "../app/components/providers/ProviderForm.vue",
  "../app/composables/useProviderForm.ts",
]
  .map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
  .join("\n");

test("保存供应商后立即清除只用于请求的密钥输入", () => {
  expect(providerSource).toContain('apiKey.value = ""');
  expect(providerSource).toContain('emit("save"');
});

test("供应商页通过原生命令加载目录并交给表单", () => {
  const page = pageSource();

  expect(page).toContain(
    'invokeCommand<CatalogProvider[]>("catalog_providers_list")',
  );
  expect(page).toContain(':catalog-providers="catalogProviders"');
  expect(providerSource).toContain(
    "catalogProviders: () => props.catalogProviders",
  );
});

test("供应商表单不提供自定义供应商入口", () => {
  expect(providerSource).not.toContain("CUSTOM_PROVIDER_TEMPLATE");
  expect(providerSource).not.toContain('label: "自定义"');
  expect(providerSource).not.toContain('providerType: "openai_compatible"');
  expect(providerSource).toContain("if (!provider && !providerTemplate.value)");
});

test("切换供应商模板时同步回填模板名称", () => {
  expect(providerSource).toContain("name.value = template.label;");
});

test("供应商表单将 Chat Completions 排在支持协议首位并包含图像生成", () => {
  expect(providerSource).toMatch(
    /const allProtocols: UpstreamProtocol\[\] = \[\s*"openai",\s*"responses",\s*"anthropic",\s*"images_generations",\s*\]/,
  );
  expect(providerSource).toContain('images_generations: "",');
  expect(providerSource).toContain(
    "const orderedUpstreamProtocols = computed(() =>",
  );
  expect(providerSource).toContain(
    'v-for="protocol in orderedUpstreamProtocols"',
  );
  expect(providerSource).toContain(
    "upstream_protocols: orderedUpstreamProtocols.value",
  );
});

test("图像生成协议不提供会产生实际调用的测试按钮", () => {
  expect(providerSource).toMatch(
    /v-if="protocol !== 'images_generations'"[\s\S]{0,260}@click="requestProtocolTest\(protocol\)"/,
  );
});

test("编辑供应商向原生层传递表单中回填的密钥", () => {
  const page = readFileSync(
    new URL("../app/pages/providers.vue", import.meta.url),
    "utf8",
  );
  expect(page).toMatch(/api_key:\s*payload\.api_key,\s*\n/);
});

test("编辑供应商将密钥可见性切换交给组件库输入框", () => {
  const relay = readFileSync(
    new URL("../app/stores/relay.ts", import.meta.url),
    "utf8",
  );

  expect(relay).toContain("api_key: string;");
  expect(providerSource).toContain('apiKey.value = provider?.api_key ?? ""');
  expect(providerSource).toContain('type="password"');
  expect(providerSource).not.toContain("ph:eye-slash");
  expect(providerSource).not.toContain("showApiKey");
  expect(providerSource).not.toContain("secret-input");
});

test("供应商表格使用组件库表格，并通过供应商 ID 执行 Ping", () => {
  const page = readFileSync(
    new URL("../app/pages/providers.vue", import.meta.url),
    "utf8",
  );
  const list = readFileSync(
    new URL("../app/components/providers/ProviderList.vue", import.meta.url),
    "utf8",
  );
  expect(page).not.toContain('"providers_discover_models"');
  expect(page).toContain('"providers_test_protocol"');
  expect(page).toContain('"providers_ping"');
  expect(page).toContain("{ providerId: provider.id }");
  expect(page).not.toMatch(/providers_ping[\\s\\S]{0,180}api_key/);
  expect(page).toContain("await loadProviders()");
  expect(list).toContain("<Table");
  expect(list).toContain("pingStatus(row.id)");
  expect(page).toContain("@ping");
  expect(list).toContain("emit('ping', row)");
  expect(list).not.toContain("PROVIDER_TEMPLATE_GROUPS");
  expect(list).not.toContain("api_key_masked");
  expect(list).not.toContain("<EmptyState");
  expect(list).toContain('empty-text="暂无供应商"');
  expect(list).toContain(':loading="loading"');
  expect(page).toContain(':loading="loadingProviders"');
});

test("供应商页不重复显示已经进入全局通知的命令错误", () => {
  expect(pageSource()).not.toContain(
    'v-if="error" class="notice notice--danger"',
  );
});

test("删除供应商使用危险确认对话框", () => {
  const page = pageSource();
  expect(page).toContain("useConfirm");
  expect(page).toContain('title: "删除供应商"');
  expect(page).toContain("该供应商及其模型将一并删除，且无法恢复。");
  expect(page).toContain('confirmText: "删除"');
  expect(page).toContain("danger: true");
  expect(page).not.toContain("if (!confirm(`删除供应商“${provider.name}”");
});

test("供应商表单回显并保存全部能力覆盖", () => {
  expect(providerSource).toContain("capabilities: ProviderCapabilities");
  expect(providerSource).toContain("protocolBaseUrls");
  expect(providerSource).toContain("tool_calls");
  expect(providerSource).toContain("max_context_tokens");
  expect(providerSource).toContain("ref<boolean | null>(null)");
  expect(pageSource()).toContain("capabilities: payload.capabilities");
});

test("供应商编辑抽屉将操作固定在 Drawer footer", () => {
  const page = pageSource();
  expect(providerSource).toContain('<form id="provider-form"');
  expect(providerSource).not.toContain("form-actions");
  expect(page).not.toContain(':show-footer="false"');
  expect(page).toContain("<template #footer>");
  expect(page).toContain('form="provider-form"');
});

test("供应商页面的页面级命令使用简短文案", () => {
  const page = pageSource();
  expect(page).toMatch(/@click="newProvider"\s*>\s*新增\s*<\/Button\s*>/);
  expect(page).toContain('pending ? "保存中..." : "保存"');
  expect(providerSource).not.toContain("<template #title>新增模型</template>");
});

test("供应商模型清单与接入点使用同一类列表工作流", () => {
  expect(providerSource).not.toContain("Popover");
  expect(providerSource).not.toContain("showAddModel");
  expect(providerSource).not.toContain("setModelPopover");
  expect(providerSource).toContain("<span>{{ models.length }} 个</span>");
  expect(providerSource).not.toContain("新增模型");
  expect(providerSource).not.toContain("model-popover");
  expect(providerSource).not.toContain("removeModel");
  expect(providerSource).not.toContain('aria-label="删除模型"');
  expect(providerSource).toContain("languageModels");
  expect(providerSource).toContain("imageGenerationModels");
  expect(providerSource).not.toContain(
    "接入点页只能选择这里已经配置的上游模型。",
  );
  expect(providerSource).not.toContain('class="model-adder"');
  expect(providerSource).toContain('class="model-group model-group--language"');
  expect(providerSource).toMatch(
    /\.model-list\s*\{[\s\S]{0,120}grid-template-columns: minmax\(0, 1fr\);/,
  );
  expect(providerSource).not.toContain(
    "grid-template-columns: repeat(2, minmax(0, 1fr));",
  );
  expect(providerSource).toContain("model-group__header");
  expect(providerSource).toContain("model-tags");
  expect(providerSource).toContain("<Tag");
  expect(providerSource).not.toContain('class="model-row"');
  expect(providerSource).not.toContain(
    "border-top: 1px solid var(--st-border-divider);",
  );
});

test("供应商表单允许覆盖各协议 Base URL", () => {
  expect(providerSource).toContain('label="Base URL"');
  expect(providerSource).toContain('v-model="protocolBaseUrls[protocol]"');
  expect(providerSource).not.toContain("默认 Base URL");
});

test("供应商操作使用带提示的图标按钮", () => {
  expect(providerSource).toMatch(
    /square\s+type="button"\s+icon="ph:flask"[\s\S]{0,180}@click="requestProtocolTest\(protocol\)"/,
  );
  expect(providerSource).not.toMatch(/icon="ph:flask"[\s\S]{0,80}size="small"/);
  expect(providerSource).not.toContain("requestDiscovery");
  expect(providerSource).not.toContain("ph:download-simple");
  expect(providerSource).toContain('aria-label="测试协议"');
  expect(providerSource).not.toContain('aria-label="获取模型"');
});

test("供应商协议按目录只读展示且地址可覆盖", () => {
  expect(providerSource).toMatch(
    /<span class="protocol-label">\s*\{\{ protocolLabel\(protocol\) \}\}/,
  );
  expect(providerSource).not.toMatch(
    /<Select[\s\S]{0,120}v-model="upstreamProtocols"/,
  );
  expect(providerSource).toMatch(
    /<Input[\s\S]{0,120}v-model="protocolBaseUrls\[protocol\]"/,
  );
});

test("供应商与活动表共用协议 Tag 颜色", () => {
  const providerList = readFileSync(
    new URL("../app/components/providers/ProviderList.vue", import.meta.url),
    "utf8",
  );
  const requestTable = readFileSync(
    new URL("../app/components/activity/RequestTable.vue", import.meta.url),
    "utf8",
  );

  expect(providerList).toContain("protocolTagVariant");
  expect(providerList).toContain("<Tag");
  expect(requestTable).toContain("protocolTagVariant");
  expect(requestTable).toContain("<Tag");
  expect(requestTable).not.toContain("function protocolLabel");
});

test("供应商模型字符串 ID 关联完整目录对象并以显示名称生成选项", () => {
  const languageModel = {
    id: "chat-model",
    display_name: "Chat Model",
    description: "完整模型对象",
  };
  setModelCatalog({
    language_models: [languageModel],
    image_generation_models: [],
    providers: [],
  });

  expect(providerModelOptions(["chat-model"])).toEqual([
    { value: "chat-model", label: "Chat Model", model: languageModel },
  ]);
  expect(providerModelOptions(["chat-model", "image-model"])).toEqual([
    { value: "chat-model", label: "Chat Model", model: languageModel },
    { value: "image-model", label: "image-model", model: undefined },
  ]);
});

test("供应商模型清单使用目录显示名但保留模型 ID", () => {
  expect(providerSource).toContain("providerModelOptions");
  expect(providerSource).toContain("modelOption.label");
  expect(providerSource).toContain("modelOption.value");
  expect(providerSource).not.toMatch(/v-for="model in languageModels"/);
  expect(providerSource).not.toMatch(/v-for="model in imageGenerationModels"/);
});

function pageSource() {
  return readFileSync(
    new URL("../app/pages/providers.vue", import.meta.url),
    "utf8",
  );
}
