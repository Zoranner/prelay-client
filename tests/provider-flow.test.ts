import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const providerSource = readFileSync(
  new URL("../app/components/providers/ProviderForm.vue", import.meta.url),
  "utf8",
);

test("保存供应商后立即清除只用于请求的密钥输入", () => {
  expect(providerSource).toContain('apiKey.value = ""');
  expect(providerSource).toContain('emit("save"');
});

test("新增供应商默认使用套餐服务首项 GoToken 套餐", () => {
  expect(providerSource).toContain("PROVIDER_TEMPLATE_GROUPS[0]?.options[0]");
  expect(providerSource).toContain(
    'providerTemplate.value = template?.value ?? "gotoken"',
  );
  expect(providerSource).toContain(
    'provider?.name ?? template?.label ?? "GoToken 套餐"',
  );
});

test("切换供应商模板时同步回填模板名称", () => {
  expect(providerSource).toContain("name.value = template.label;");
});

test("供应商表单将 Chat Completions 排在支持协议首位并包含图像生成", () => {
  expect(providerSource).toContain(
    'const allProtocols: UpstreamProtocol[] = ["openai", "responses", "anthropic", "images_generations"]',
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

test("编辑供应商回填密钥并支持显示或隐藏", () => {
  const relay = readFileSync(
    new URL("../app/stores/relay.ts", import.meta.url),
    "utf8",
  );

  expect(relay).toContain("api_key: string;");
  expect(providerSource).toContain('apiKey.value = provider?.api_key ?? ""');
  expect(providerSource).toContain("ph:eye-slash");
  expect(providerSource).toContain("showApiKey");
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
  expect(page).toContain('"providers_discover_models"');
  expect(page).toContain('"providers_test_protocol"');
  expect(page).toContain('"providers_ping"');
  expect(page).toContain("{ providerId: provider.id }");
  expect(page).not.toMatch(/providers_ping[\\s\\S]{0,180}api_key/);
  expect(page).toContain("await loadProviders()");
  expect(list).toContain("<Table");
  expect(list).toContain("pingStatus(row.id)");
  expect(page).toContain("@ping");
  expect(list).toContain("emit('ping', row)");
  expect(list).toContain("套餐");
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
  expect(page).toMatch(/@click="newProvider">新增<\/Button>/);
  expect(page).toContain('pending ? "保存中..." : "保存"');
  expect(providerSource).toContain("<template #title>新增模型</template>");
});

test("供应商模型清单与接入点使用同一类列表工作流", () => {
  expect(providerSource).toContain("Popover");
  expect(providerSource).toContain("showAddModel");
  expect(providerSource).toContain("setModelPopover");
  expect(providerSource).toContain("<span>{{ models.length }} 个</span>");
  expect(providerSource).toContain("新增模型");
  expect(providerSource).toContain("model-popover");
  expect(providerSource).not.toContain(
    "接入点页只能选择这里已经配置的上游模型。",
  );
  expect(providerSource).not.toContain('class="model-adder"');
  expect(providerSource).toContain(
    "border-top: 1px solid var(--st-border-divider);",
  );
  expect(providerSource).not.toContain("border: 1px solid var(--st-border);");
});

test("供应商表单将基础上游地址作为唯一的 Base URL", () => {
  expect(providerSource).toContain('label="Base URL"');
  expect(providerSource).toContain(
    ":placeholder=\"baseUrl || '填写协议地址'\"",
  );
  expect(providerSource).not.toContain("默认 Base URL");
});

test("供应商操作使用带提示的图标按钮", () => {
  expect(providerSource).toMatch(
    /icon="ph:plugs-connected"[\s\S]{0,180}@click="requestProtocolTest\(protocol\)"/,
  );
  expect(providerSource).toMatch(
    /icon="ph:download-simple"[\s\S]{0,180}@click="requestDiscovery"/,
  );
  expect(providerSource).toMatch(/@click="requestDiscovery"\s*>\s*获取/);
  expect(providerSource).toContain('aria-label="测试协议"');
  expect(providerSource).toContain('aria-label="获取模型"');
});

test("供应商支持协议使用不同颜色的 Tag 多选", () => {
  expect(providerSource).toContain("const protocolOptions = allProtocols.map(");
  expect(providerSource).toContain("tagVariant:");
  expect(providerSource).toMatch(
    /<Select[\s\S]{0,120}v-model="upstreamProtocols"[\s\S]{0,180}multiple/,
  );
  expect(providerSource).not.toContain('class="protocol-checks"');
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

function pageSource() {
  return readFileSync(
    new URL("../app/pages/providers.vue", import.meta.url),
    "utf8",
  );
}
