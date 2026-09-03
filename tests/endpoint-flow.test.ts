import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const endpointPage = readFileSync(
  new URL("../app/pages/endpoints.vue", import.meta.url),
  "utf8",
);
const endpointForm = readFileSync(
  new URL("../app/components/endpoints/EndpointForm.vue", import.meta.url),
  "utf8",
);
const endpointList = readFileSync(
  new URL("../app/components/endpoints/EndpointList.vue", import.meta.url),
  "utf8",
);

test("接入点页面提供模型映射和 Token 重置", () => {
  expect(endpointPage).toContain('"endpoints_save"');
  expect(endpointPage).toContain('"endpoints_regenerate_token"');
  expect(endpointPage).not.toContain("https://relay.rd.kim");
  expect(endpointPage).toContain("upstream_model");
  expect(endpointPage).not.toContain("/proxy");
  expect(endpointPage).toContain("useConfirm");
  expect(endpointPage).toContain('title: "重置 API Token"');
  expect(endpointPage).toContain('confirmText: "重置"');
  expect(endpointPage).toContain("danger: true");
  expect(endpointPage).toContain("现有工具将立即失效。");
  expect(endpointPage).not.toContain("if (!confirm(`重置“${item.name}”");
  expect(endpointPage).toContain('title: "删除接入点"');
  expect(endpointPage).toContain("删除后无法恢复。");
  expect(endpointPage).not.toContain("if (!confirm(`删除接入点“${item.name}”");
});

test("接入点模型映射只允许选择已保存模型的供应商", () => {
  expect(endpointForm).toContain("availableProviders");
  expect(endpointForm).toContain("provider.models.length > 0");
  expect(endpointForm).toContain("function modelsForProvider");
  expect(endpointForm).toContain("providerOptions");
});

test("接入点允许多个供应商映射为同一个对外模型名", () => {
  expect(endpointForm).not.toContain("请使用不同的对外模型名。");
  expect(endpointForm).not.toContain("接入点模型名重复");
});

test("接入点模型列表按对外模型分组展示供应商候选", () => {
  expect(endpointForm).toContain("const modelGroups");
  expect(endpointForm).toContain('v-for="group in modelGroups"');
  expect(endpointForm).toContain("group.mappings.length");
  expect(endpointForm).toContain("group.mappings");
});

test("接入点模型从列表上下文新增而非底部统一编辑器", () => {
  expect(endpointForm).toContain("Popover");
  expect(endpointForm).toContain("model-popover");
  expect(endpointForm).toContain("新增模型");
  expect(endpointForm).toContain("新增供应商");
  expect(endpointForm).toContain('align="right"');
  expect(endpointForm).toContain('size="large"');
  expect(endpointForm.match(/<template #footer>/g)?.length).toBe(2);
  expect(endpointForm).toContain("无可用模型");
  expect(endpointForm).not.toContain("无可用上游模型");
  expect(endpointForm.match(/>确认<\/Button/g)?.length).toBe(2);
  expect(endpointForm).not.toContain('class="model-editor"');
  expect(endpointForm).not.toContain('class="add-model-form"');
  expect(endpointForm).not.toContain('class="add-provider-form"');
});

test("接入点模型分组新增供应商时排除已绑定的上游模型", () => {
  expect(endpointForm).toContain("function availableUpstreamModels");
  expect(endpointForm).toContain("mapping.model.provider_id === providerId");
  expect(endpointForm).toContain("mapping.model.upstream_model");
  expect(endpointForm).toMatch(
    /:options="\s*upstreamModelOptions\(newProviderForm\.provider_id, group\)\s*"/,
  );
  expect(endpointForm).toContain(
    '@change="selectProvider(newProviderForm, group)"',
  );
  expect(endpointForm).toContain(':options="providerOptions"');
  expect(endpointForm).not.toContain("providerOptionsForGroup");
});

test("新建接入点保持旧网页的名称和模型配置内容", () => {
  expect(endpointForm).not.toContain('<select v-model="protocol"');
  expect(endpointForm).not.toContain('<span class="field__label">协议</span>');
  expect(endpointPage).not.toContain("请先在供应商页面配置至少一个模型。");
  expect(endpointPage).not.toContain("hasProviderModels");
  expect(endpointPage).toContain('@click="createEndpoint"');
});

test("接入点页面的页面级命令使用简短文案", () => {
  expect(endpointPage).toContain('@click="createEndpoint"');
  expect(endpointPage).toContain("新增");
  expect(endpointPage).toContain('pending ? "保存中..." : "保存"');
});

test("接入点表格收缩长名称和 Token 列而保持操作列可用", () => {
  expect(endpointList).toContain('from "@stellar/ui"');
  expect(endpointList).toContain("<Table");
  expect(endpointList).toContain("endpoint-name");
  expect(endpointList).toContain('fixed: "right"');
  expect(endpointList).toContain("ellipsis: true");
  expect(endpointList).not.toContain("<EmptyState");
  expect(endpointList).toContain('empty-text="暂无接入点"');
  expect(endpointList).toContain(':loading="pending"');
  expect(endpointList).toContain(
    '{ key: "name", title: "名称", width: 280, ellipsis: true }',
  );
  expect(endpointList).toContain('icon-position="right"');
  expect(endpointList).toContain('icon="ph:key"');
  expect(endpointList).not.toContain('icon="ph:arrows-clockwise"');
});

test("接入点表格以模型标签展示前三个模型并汇总其余数量", () => {
  expect(endpointList).not.toContain('key: "protocol"');
  expect(endpointList).not.toContain("protocolLabel");
  expect(endpointList).toContain('layout="auto"');
  expect(endpointList).toContain("row.models.slice(0, 3)");
  expect(endpointList).toContain("+{{ row.models.length - 3 }}");
  expect(endpointList).toContain("<Tag");
  expect(endpointList).not.toContain("<Badge");
});

test("接入点编辑抽屉将操作固定在 Drawer footer", () => {
  expect(endpointForm).toContain('<form id="endpoint-form"');
  expect(endpointForm).not.toContain("form-actions");
  expect(endpointPage).not.toContain(':show-footer="false"');
  expect(endpointPage).toContain("<template #footer>");
  expect(endpointPage).toContain('form="endpoint-form"');
});

test("接入点模型显示目录或服务端显示名并保留模型 ID", () => {
  expect(endpointForm).toContain("group.displayName");
  expect(endpointForm).toContain("modelCatalogLabel");
  expect(endpointForm).toContain("model.display_name");
  expect(endpointForm).toContain("model_name: upstream_model");
  expect(endpointList).toContain("model.display_name");
  expect(endpointList).toContain("modelCatalogLabel");
});
