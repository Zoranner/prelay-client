<script setup lang="ts">
import {
  Button,
  FormField,
  Input,
  Popover,
  Select,
  useNotification,
} from "stellar-ui";
import type {
  Provider,
  ProviderCapabilities,
  UpstreamProtocol,
} from "~/stores/relay";
import {
  PROVIDER_TEMPLATE_GROUPS,
  protocolLabel,
  protocolTagVariant,
  providerTemplateForType,
} from "~/utils/providerTemplates";
import {
  getProviderOperationFeedback,
  type ProviderOperationResult,
} from "~/utils/providerOperations";

const props = defineProps<{
  provider?: Provider | null;
  pending?: boolean;
  discoverModels: (
    input: ProviderOperationInput,
  ) => Promise<ProviderOperationResult>;
  testProtocol: (
    input: ProviderOperationInput,
  ) => Promise<ProviderOperationResult>;
}>();

const emit = defineEmits<{
  save: [
    payload: {
      id?: string;
      name: string;
      provider_type: string;
      base_url: string;
      api_key: string;
      capabilities: ProviderCapabilities;
      models: string[];
    },
  ];
  cancel: [];
  "dirty-change": [dirty: boolean];
}>();

type ProviderOperationInput = {
  provider_type: string;
  base_url: string;
  api_key: string;
  protocol?: UpstreamProtocol;
  model?: string;
};

const allProtocols: UpstreamProtocol[] = ["openai", "responses", "anthropic"];
const protocolOptions = allProtocols.map((protocol) => ({
  value: protocol,
  label: protocolLabel(protocol),
  tagVariant: protocolTagVariant(protocol),
}));
const providerTemplate = ref("custom");
const name = ref("");
const providerType = ref("openai_compatible");
const baseUrl = ref("");
const apiKey = ref("");
const models = ref<string[]>([]);
const modelDraft = ref("");
const upstreamProtocols = ref<UpstreamProtocol[]>([]);
const orderedUpstreamProtocols = computed(() =>
  allProtocols.filter((protocol) => upstreamProtocols.value.includes(protocol)),
);
const protocolBaseUrls = reactive<Record<UpstreamProtocol, string>>({
  responses: "",
  openai: "",
  anthropic: "",
});
const toolCalls = ref<boolean | null>(null);
const reasoning = ref<boolean | null>(null);
const toolChoice = ref<boolean | null>(null);
const parallelToolCalls = ref<boolean | null>(null);
const systemMessages = ref<boolean | null>(null);
const structuredOutputs = ref<boolean | null>(null);
const streamingUsage = ref<boolean | null>(null);
const maxContextTokens = ref<number | null>(null);
const maxOutputTokens = ref<number | null>(null);
const preservedCapabilities = ref<ProviderCapabilities>({});
const notifications = useNotification();
const showApiKey = ref(false);
const showAddModel = ref(false);
const defaultProviderTemplate = PROVIDER_TEMPLATE_GROUPS[0]?.options[0];
const providerTemplateOptions = PROVIDER_TEMPLATE_GROUPS.flatMap((group) =>
  group.options.map((option) => ({
    label: `${group.label} - ${option.label}`,
    value: option.value,
  })),
);
let initialDraft = "";

function serializeDraft() {
  return JSON.stringify({
    name: name.value,
    providerType: providerType.value,
    baseUrl: baseUrl.value,
    apiKey: apiKey.value,
    models: models.value,
    upstreamProtocols: upstreamProtocols.value,
    protocolBaseUrls,
    toolCalls: toolCalls.value,
    reasoning: reasoning.value,
    toolChoice: toolChoice.value,
    parallelToolCalls: parallelToolCalls.value,
    systemMessages: systemMessages.value,
    structuredOutputs: structuredOutputs.value,
    streamingUsage: streamingUsage.value,
    maxContextTokens: maxContextTokens.value,
    maxOutputTokens: maxOutputTokens.value,
  });
}

watch(
  () => props.provider,
  (provider) => {
    const template = provider
      ? providerTemplateForType(provider.provider_type)
      : defaultProviderTemplate;
    providerTemplate.value = template?.value ?? "gotoken";
    name.value = provider?.name ?? template?.label ?? "GoToken 套餐";
    providerType.value =
      provider?.provider_type ?? template?.providerType ?? "gotoken";
    baseUrl.value = provider?.base_url ?? template?.baseUrl ?? "";
    apiKey.value = provider?.api_key ?? "";
    showApiKey.value = false;
    showAddModel.value = false;
    modelDraft.value = "";
    models.value = provider?.models.map((model) => model.model_name) ?? [];
    upstreamProtocols.value = (
      provider?.capabilities?.upstream_protocols ??
      template?.protocols ??
      []
    ).filter(isProtocol);
    for (const protocol of allProtocols) {
      protocolBaseUrls[protocol] =
        provider?.capabilities?.protocol_base_urls?.[protocol] ??
        template?.protocolBaseUrls[protocol] ??
        "";
    }
    preservedCapabilities.value = provider?.capabilities ?? {};
    toolCalls.value = provider?.capabilities?.tool_calls ?? null;
    reasoning.value = provider?.capabilities?.reasoning ?? null;
    toolChoice.value = provider?.capabilities?.tool_choice ?? null;
    parallelToolCalls.value =
      provider?.capabilities?.parallel_tool_calls ?? null;
    systemMessages.value = provider?.capabilities?.system_messages ?? null;
    structuredOutputs.value =
      provider?.capabilities?.structured_outputs ?? null;
    streamingUsage.value = provider?.capabilities?.streaming_usage ?? null;
    maxContextTokens.value = provider?.capabilities?.max_context_tokens ?? null;
    maxOutputTokens.value = provider?.capabilities?.max_output_tokens ?? null;
    initialDraft = serializeDraft();
    emit("dirty-change", false);
  },
  { immediate: true },
);

watch(
  serializeDraft,
  (draft) => emit("dirty-change", draft !== initialDraft),
);

function isProtocol(value: string): value is UpstreamProtocol {
  return allProtocols.includes(value as UpstreamProtocol);
}

function selectProviderTemplate() {
  const template = PROVIDER_TEMPLATE_GROUPS.flatMap(
    (group) => group.options,
  ).find((item) => item.value === providerTemplate.value);
  if (!template) return;
  name.value = template.label;
  providerType.value = template.providerType;
  baseUrl.value = template.baseUrl;
  upstreamProtocols.value = [...template.protocols];
  for (const protocol of allProtocols) {
    protocolBaseUrls[protocol] = template.protocolBaseUrls[protocol] ?? "";
  }
}

function addModel() {
  const model = modelDraft.value.trim();
  if (!model || models.value.includes(model)) return;
  models.value.push(model);
  modelDraft.value = "";
  showAddModel.value = false;
}

function setModelPopover(open: boolean) {
  showAddModel.value = open;
  if (!open) modelDraft.value = "";
}

function removeModel(model: string) {
  models.value = models.value.filter((item) => item !== model);
}

async function requestDiscovery() {
  if (!apiKey.value.trim()) {
    notifications.danger("请填写上游 API Key。", { title: "连接配置不完整" });
    return;
  }
  const result = await props.discoverModels(operationInput());
  if (!result.ok) {
    notifications.danger(result.error ?? "请检查连接信息和 API Key。", {
      title: "模型获取失败",
    });
    return;
  }
  models.value = result.models ?? [];
  notifications.success(`共 ${models.value.length} 个模型。`, {
    title: "模型已获取",
  });
}

async function requestProtocolTest(protocol: UpstreamProtocol) {
  if (!apiKey.value.trim()) {
    notifications.danger("请填写上游 API Key。", { title: "连接配置不完整" });
    return;
  }
  const result = await props.testProtocol(operationInput(protocol));
  const feedback = getProviderOperationFeedback(result);
  notifications.notify({
    type: feedback.success ? "success" : "danger",
    title: feedback.message,
    message: feedback.metrics ?? "",
  });
}

function operationInput(protocol?: UpstreamProtocol): ProviderOperationInput {
  return {
    provider_type: providerType.value,
    base_url:
      protocolBaseUrls[protocol ?? "openai"].trim() || baseUrl.value.trim(),
    api_key: apiKey.value.trim(),
    ...(protocol ? { protocol } : {}),
    ...(models.value[0] ? { model: models.value[0] } : {}),
  };
}

function submit() {
  if (!name.value.trim() || !baseUrl.value.trim()) {
    notifications.danger("请填写名称和 Base URL。", {
      title: "连接配置不完整",
    });
    return;
  }
  if (!props.provider && !apiKey.value.trim()) {
    notifications.danger("请填写上游 API Key。", { title: "连接配置不完整" });
    return;
  }
  emit("save", {
    ...(props.provider ? { id: props.provider.id } : {}),
    name: name.value.trim(),
    provider_type: providerType.value,
    base_url: baseUrl.value.trim(),
    api_key: apiKey.value,
    capabilities: {
      ...preservedCapabilities.value,
      upstream_protocols: orderedUpstreamProtocols.value,
      protocol_base_urls: Object.fromEntries(
        allProtocols.map((protocol) => [
          protocol,
          protocolBaseUrls[protocol].trim() || null,
        ]),
      ),
      tool_calls: toolCalls.value,
      reasoning: reasoning.value,
      tool_choice: toolChoice.value,
      parallel_tool_calls: parallelToolCalls.value,
      system_messages: systemMessages.value,
      structured_outputs: structuredOutputs.value,
      streaming_usage: streamingUsage.value,
      max_context_tokens: maxContextTokens.value,
      max_output_tokens: maxOutputTokens.value,
    },
    models: models.value,
  });
  apiKey.value = "";
}
</script>

<template>
  <form id="provider-form" class="provider-form" @submit.prevent="submit">
    <section class="form-section">
      <h3>连接配置</h3>
      <div class="form-fields">
        <Select
          v-model="providerTemplate"
          label="供应商"
          :options="providerTemplateOptions"
          @change="selectProviderTemplate"
        />
        <Input v-model="name" label="名称" autocomplete="off" />
        <FormField label="API Key">
          <div class="secret-input">
            <Input
              v-model="apiKey"
              class="secret-input__field"
              :type="showApiKey ? 'text' : 'password'"
              placeholder="填写上游 API Key"
            />
            <Button
              square
              size="small"
              :icon="showApiKey ? 'ph:eye-slash' : 'ph:eye'"
              :aria-label="showApiKey ? '隐藏 API Key' : '显示 API Key'"
              :title="showApiKey ? '隐藏 API Key' : '显示 API Key'"
              @click="showApiKey = !showApiKey"
            />
          </div>
        </FormField>
        <Input v-model="baseUrl" label="Base URL" type="url" />
        <Select
          v-model="upstreamProtocols"
          multiple
          label="支持协议"
          placeholder="选择支持协议"
          :options="protocolOptions"
        />
        <div class="protocol-urls">
          <div
            v-for="protocol in orderedUpstreamProtocols"
            :key="protocol"
            class="protocol-url-row"
          >
            <span>{{ protocolLabel(protocol) }}</span>
            <Input
              v-model="protocolBaseUrls[protocol]"
              :placeholder="baseUrl || '填写协议地址'"
            />
            <Button
              square
              type="button"
              size="small"
              icon="ph:plugs-connected"
              aria-label="测试协议"
              title="测试协议"
              :disabled="pending"
              @click="requestProtocolTest(protocol)"
            />
          </div>
        </div>
      </div>
    </section>

    <section class="form-section">
      <div class="section-header">
        <h3>模型清单</h3>
        <div class="section-header__actions">
          <span>{{ models.length }} 个</span>
          <Button
            type="button"
            size="small"
            icon="ph:download-simple"
            aria-label="获取模型"
            title="获取模型"
            :disabled="pending"
            @click="requestDiscovery"
            >获取</Button
          >
          <Popover
            v-model="showAddModel"
            position="bottom"
            align="right"
            size="large"
            @update:model-value="setModelPopover"
          >
            <Button size="small" type="button" variant="primary" icon="ph:plus">
              新增
            </Button>
            <template #title>新增模型</template>
            <template #content>
              <div class="model-popover">
                <Input
                  v-model="modelDraft"
                  label="上游模型"
                  placeholder="kimi-k2-0711-preview"
                  @enter="addModel"
                />
              </div>
            </template>
            <template #footer>
              <Button
                variant="primary"
                type="button"
                :disabled="pending"
                @click="addModel"
                >确认</Button
              >
            </template>
          </Popover>
        </div>
      </div>
      <div class="model-list">
        <div v-for="model in models" :key="model" class="model-row">
          <code>{{ model }}</code>
          <Button
            square
            size="small"
            variant="danger"
            icon="ph:trash"
            aria-label="删除模型"
            title="删除模型"
            @click="removeModel(model)"
          />
        </div>
        <p v-if="!models.length" class="empty-text">暂无模型。</p>
      </div>
    </section>
  </form>
</template>

<style scoped>
.provider-form,
.form-section,
.form-fields,
.model-list {
  display: grid;
  gap: var(--spacing-lg);
}
.provider-form {
  padding: var(--spacing-lg);
}
.form-section {
  margin: 0;
}
.form-section h3,
.form-section p {
  margin: 0;
}
.form-section h3 {
  color: var(--st-text-primary);
  font-size: 15px;
}
.form-section p,
.empty-text {
  color: var(--st-text-secondary);
}
.secret-input,
.protocol-url-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
.secret-input__field {
  flex: 1;
}
.protocol-urls {
  display: grid;
  gap: var(--spacing-sm);
}
.protocol-url-row > span {
  width: 160px;
  color: var(--st-text-secondary);
  white-space: nowrap;
}
.protocol-url-row > :nth-child(2) {
  flex: 1;
}
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--spacing-md);
}
.section-header__actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
.section-header__actions > span {
  color: var(--st-text-secondary);
}
.model-list {
  gap: var(--spacing-sm);
}
.model-row {
  display: flex;
  min-width: 0;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
  border-top: 1px solid var(--st-border-divider);
  padding: var(--spacing-sm) 0 0 var(--spacing-md);
}
.model-row code {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.model-popover {
  display: grid;
  gap: var(--spacing-md);
}
@media (max-width: 720px) {
  .protocol-url-row {
    align-items: stretch;
    flex-direction: column;
  }
  .protocol-url-row > span {
    width: auto;
  }
}
</style>
