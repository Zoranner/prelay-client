import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { setModelCatalog } from "../app/stores/modelCatalog";
import { providerModelOptions } from "../app/stores/modelCatalog";

const providerSource = [
  "../app/components/providers/Form.vue",
  "../app/composables/providerForm.ts",
]
  .map((path) => readFileSync(new URL(path, import.meta.url), "utf8"))
  .join("\n");

function pageSource() {
  return readFileSync(
    new URL("../app/pages/providers.vue", import.meta.url),
    "utf8",
  );
}
const providerListSource = readFileSync(
  new URL("../app/components/providers/List.vue", import.meta.url),
  "utf8",
);
const providerSharingDrawerUrl = new URL(
  "../app/components/providers/sharing/Drawer.vue",
  import.meta.url,
);
const providerSharingDrawerSource = existsSync(providerSharingDrawerUrl)
  ? readFileSync(providerSharingDrawerUrl, "utf8")
  : "";

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

test("供应商表单按目录展示协议且不提交协议集合", () => {
  expect(providerSource).toMatch(
    /const allProtocols: UpstreamProtocol\[\] = \[\s*"openai",\s*"responses",\s*"images_generations",\s*\]/,
  );
  expect(providerSource).not.toContain('"anthropic"');
  expect(providerSource).not.toContain("anthropic:");
  expect(providerSource).toContain("const currentTemplate = computed(");
  expect(providerSource).toContain(
    "const orderedUpstreamProtocols = computed(() =>",
  );
  expect(providerSource).toContain(
    'v-for="protocol in orderedUpstreamProtocols"',
  );
  expect(providerSource).not.toContain("upstream_protocols:");
  expect(providerSource).toContain("protocol_base_urls:");
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
    new URL("../app/types/relay.ts", import.meta.url),
    "utf8",
  );

  expect(relay).toContain("api_key: string;");
  expect(providerSource).toContain("provider?.credential?.api_key");
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
    new URL("../app/components/providers/List.vue", import.meta.url),
    "utf8",
  );
  expect(page).not.toContain('"providers_discover_models"');
  expect(page).toContain('"providers_test_protocol"');
  expect(page).toContain('"providers_ping"');
  expect(page).toContain("{ providerId: provider.id }");
  expect(page).not.toMatch(/providers_ping[\\s\\S]{0,180}api_key/);
  expect(page).toContain("await loadProviders()");
  expect(list).toContain("<Table");
  expect(list).toContain("pingStatus(provider.id)");
  expect(page).toContain("@ping");
  expect(list).toContain("emit('ping', row)");
  expect(list).not.toContain("PROVIDER_TEMPLATE_GROUPS");
  expect(list).not.toContain("api_key_masked");
  expect(list).not.toContain("<EmptyState");
  expect(list).toContain('empty-text="暂无供应商"');
  expect(list).toContain(':loading="loading"');
  expect(list).toContain('layout="fixed"');
  expect(page).toContain(':loading="loadingProviders"');
});

test("供应商列表展示创建人和额度列，不再展示可见范围", () => {
  expect(providerListSource).toContain('key: "owner"');
  expect(providerListSource).toContain('key: "quota"');
  expect(providerListSource).toContain("owner_display_name");
  expect(providerListSource).toContain('title: "剩余额度"');
  expect(providerListSource).toContain("ProviderQuotaCell");
  expect(providerListSource).toContain("identityAvatarSrc");
  expect(providerListSource).not.toContain('key: "visibility"');
  expect(providerListSource).not.toContain("ProviderVisibilityScope");
  expect(providerListSource).not.toContain('key: "models"');
  expect(providerListSource).not.toContain("modelCatalogProviderModels");
  expect(providerListSource).not.toContain("待新增");
  expect(providerListSource).not.toContain('key: "usage"');
  expect(providerListSource).not.toContain("使用统计");
  expect(pageSource()).toContain(':quota-by-provider="quotaByProvider"');
});

test("非创建人供应商保留连通性测试，禁用编辑、删除和协议测试", () => {
  const page = pageSource();

  expect(providerListSource).toContain(':disabled="!row.isOwn"');
  expect(providerListSource).toContain(
    `:disabled="row.ping.label === '检查中'"`,
  );
  expect(page).not.toContain("can_manage");
  expect(page).toContain(
    "void Promise.all(providers.value.map(pingProvider));",
  );
  expect(providerListSource).toContain("emit('share', row)");
  expect(page).toContain("isOwnProvider(editingProvider,");
  expect(page).toContain(':retired="editingRetired"');
  expect(page).toMatch(
    /:can-test="[\s\S]{0,80}isOwnProvider\(editingProvider, currentIdentityId\)/,
  );
  expect(providerSharingDrawerSource).not.toContain("can_manage");
});

test("共享抽屉用一句话编辑额度规则", () => {
  const ruleEditor = readFileSync(
    new URL(
      "../app/components/providers/sharing/RuleEditor.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const rule = readFileSync(
    new URL("../app/components/providers/sharing/rule.ts", import.meta.url),
    "utf8",
  );

  expect(providerSharingDrawerSource).toContain('label: "不限额"');
  expect(providerSharingDrawerSource).toContain('label: "限额"');
  expect(providerSharingDrawerSource).toContain("RuleEditor");
  expect(ruleEditor).toContain("每天");
  expect(ruleEditor).toContain("亿");
  expect(ruleEditor).not.toContain("清零");
  expect(rule).toContain('label: "周内累积"');
  expect(rule).toContain('label: "月内累积"');
  expect(rule).not.toContain("按日累积");
  expect(providerSharingDrawerSource).toContain("save-sharing");
  expect(providerSharingDrawerSource).not.toContain("providers_usage_get");
});

test("供应商页面通过 Tauri commands 加载额度并保存共享", () => {
  const page = pageSource();
  const sharing = readFileSync(
    new URL("../app/composables/providerSharing.ts", import.meta.url),
    "utf8",
  );

  expect(sharing).toContain(
    'invokeCommand<ProviderSharing>("providers_sharing_get"',
  );
  expect(sharing).toContain(
    'invokeCommand<IdentityDirectoryEntry[]>("identity_directory_list")',
  );
  expect(sharing).toContain('"providers_sharing_save"');
  expect(sharing).not.toContain("providers_usage_get");
  expect(sharing).toContain("await options.loadProviders()");
  expect(page).toContain("invokeCommand<ProviderQuotaResponse[]>(");
  expect(page).toContain('"providers_quotas"');
  expect(page).toContain("<ProviderSharingDrawer");
  expect(page).toContain("@save-sharing");
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
  expect(providerSource).toContain(
    "<span>{{ enabledModels.length }} 个</span>",
  );
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
  expect(providerSource).toContain("<ProviderModelToggles");
  expect(providerSource).toContain("toggleModel");
  expect(providerSource).not.toContain("<Tag");
  expect(providerSource).not.toContain('class="model-row"');
  expect(providerSource).not.toContain(
    "border-top: 1px solid var(--st-border-divider);",
  );
});

test("供应商模型清单保存在供应商记录里", () => {
  const relay = readFileSync(
    new URL("../app/types/relay.ts", import.meta.url),
    "utf8",
  );
  const form = readFileSync(
    new URL("../app/composables/providerForm.ts", import.meta.url),
    "utf8",
  );
  const page = pageSource();

  expect(relay).toContain("models: string[]");
  expect(form).toContain(
    "const catalogModels = modelCatalogProviderModels(providerType)",
  );
  expect(form).toContain("enabledModels.value = provider");
  expect(form).toContain("models: [...enabledModels.value]");
  expect(page).toContain("models: payload.models");
});

test("供应商分享命令通过已认证管理 API 使用协议 DTO", () => {
  const command = readFileSync(
    new URL("../src-tauri/src/commands/providers.rs", import.meta.url),
    "utf8",
  );
  const application = readFileSync(
    new URL("../src-tauri/src/app/mod.rs", import.meta.url),
    "utf8",
  );
  const relayCommand = readFileSync(
    new URL("../app/composables/command/relay.ts", import.meta.url),
    "utf8",
  );

  expect(command).toContain("pub async fn providers_sharing_get(");
  expect(command).toContain(
    '.get(&format!("/api/providers/{provider_id}/sharing"))',
  );
  expect(command).toContain("pub async fn providers_sharing_save(");
  expect(command).toContain(
    '.patch(&format!("/api/providers/{provider_id}/sharing"), &input)',
  );
  expect(command).toContain("UpdateProviderSharingRequest");
  expect(command).toContain("ProviderSharingResponse");
  expect(command).toContain("pub async fn providers_usage_get(");
  expect(command).toContain(
    '"/api/providers/{provider_id}/usage?range={range}"',
  );
  expect(command).toContain("ProviderUsageResponse");
  expect(command).toContain("authenticated_api(&state)");

  expect(application).toContain(
    "crate::commands::providers::providers_sharing_get",
  );
  expect(application).toContain(
    "crate::commands::providers::providers_sharing_save",
  );
  expect(application).toContain(
    "crate::commands::providers::providers_usage_get",
  );
  expect(relayCommand).toContain('"providers_sharing_get"');
  expect(relayCommand).toContain('"providers_sharing_save"');
  expect(relayCommand).toContain('"providers_usage_get"');
});

test("身份目录命令只读取非敏感展示字段", () => {
  const command = readFileSync(
    new URL("../src-tauri/src/commands/identity.rs", import.meta.url),
    "utf8",
  );
  const application = readFileSync(
    new URL("../src-tauri/src/app/mod.rs", import.meta.url),
    "utf8",
  );
  const relayCommand = readFileSync(
    new URL("../app/composables/command/relay.ts", import.meta.url),
    "utf8",
  );

  expect(command).toContain("pub async fn identity_directory_list(");
  expect(command).toContain('.get("/api/identities")');
  expect(command).toContain("IdentityDirectoryEntry");
  expect(command).toContain("authenticated_api(&state)");
  const directoryCommand = command.slice(
    command.indexOf("pub async fn identity_directory_list("),
  );
  expect(directoryCommand).not.toContain("credential");
  expect(application).toContain(
    "crate::commands::identity::identity_directory_list",
  );
  expect(relayCommand).toContain('"identity_directory_list"');
});

test("供应商表单的协议地址由目录声明键只读、其余可覆盖", () => {
  expect(providerSource).toContain('label="Base URL"');
  expect(providerSource).toContain('v-model="protocolBaseUrls[protocol]"');
  expect(providerSource).toContain("protocolBaseUrls[protocol].trim() || null");
  expect(providerSource).toContain("isProtocolManaged(protocol)");
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

test("供应商协议按目录只读展示，目录声明的地址由目录管理", () => {
  expect(providerSource).toMatch(
    /<span class="protocol-label">\s*\{\{ protocolLabel\(protocol\) \}\}/,
  );
  expect(providerSource).not.toMatch(
    /<Select[\s\S]{0,120}v-model="upstreamProtocols"/,
  );
  expect(providerSource).toMatch(
    /<Input[\s\S]{0,120}v-model="protocolBaseUrls\[protocol\]"/,
  );
  expect(providerSource).toContain("isProtocolManaged(protocol)");
  expect(providerSource).toContain("managedProtocols");
});

test("活动表使用共用的协议 Tag 颜色", () => {
  const requestTable = readFileSync(
    new URL("../app/components/activity/Table.vue", import.meta.url),
    "utf8",
  );

  expect(requestTable).toContain("protocolTagPalette");
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
