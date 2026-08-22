<script setup lang="ts">
import {
  Button,
  Checkbox,
  FormField,
  Input,
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
}>();

type ProviderOperationInput = {
  provider_type: string;
  base_url: string;
  api_key: string;
  protocol?: UpstreamProtocol;
  model?: string;
};

const allProtocols: UpstreamProtocol[] = ["openai", "responses", "anthropic"];
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
const defaultProviderTemplate = PROVIDER_TEMPLATE_GROUPS[0]?.options[0];
const providerTemplateOptions = PROVIDER_TEMPLATE_GROUPS.flatMap((group) =>
  group.options.map((option) => ({
    label: `${group.label} - ${option.label}`,
    value: option.value,
  })),
);

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
  },
  { immediate: true },
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
    notifications.danger("请填写名称和默认 Base URL。", {
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
        <Input v-model="baseUrl" label="默认 Base URL" type="url" />
        <FormField label="支持协议">
          <div class="protocol-checks">
            <Checkbox
              v-for="protocol in allProtocols"
              :key="protocol"
              v-model="upstreamProtocols"
              :value="protocol"
              :label="protocolLabel(protocol)"
            />
          </div>
        </FormField>
        <div class="protocol-urls">
          <div
            v-for="protocol in orderedUpstreamProtocols"
            :key="protocol"
            class="protocol-url-row"
          >
            <span>{{ protocolLabel(protocol) }}</span>
            <Input
              v-model="protocolBaseUrls[protocol]"
              placeholder="留空使用默认 Base URL"
            />
            <Button
              type="button"
              size="small"
              :disabled="pending"
              @click="requestProtocolTest(protocol)"
              >测试协议</Button
            >
          </div>
        </div>
      </div>
    </section>

    <section class="form-section">
      <div class="section-header">
        <div>
          <h3>模型清单</h3>
          <p>接入点页只能选择这里已经配置的上游模型。</p>
        </div>
        <Button
          type="button"
          size="small"
          :disabled="pending"
          @click="requestDiscovery"
          >获取模型</Button
        >
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
      <div class="model-adder">
        <Input
          v-model="modelDraft"
          label="上游模型"
          placeholder="kimi-k2-0711-preview"
          @enter="addModel"
        />
        <Button variant="primary" type="button" @click="addModel"
          >新增</Button
        >
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
.form-section {
  margin: 0;
  border: 1px solid var(--st-border);
  padding: var(--spacing-lg);
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
.protocol-url-row,
.section-header,
.model-row,
.model-adder {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
.secret-input__field {
  flex: 1;
}
.protocol-checks {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--spacing-sm);
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
  justify-content: space-between;
  align-items: flex-start;
}
.model-list {
  gap: var(--spacing-sm);
}
.model-row {
  justify-content: space-between;
  border: 1px solid var(--st-border);
  padding: var(--spacing-sm) var(--spacing-md);
}
.model-row code {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.model-adder {
  align-items: flex-end;
}
.model-adder > :first-child {
  flex: 1;
}
@media (max-width: 720px) {
  .protocol-checks {
    grid-template-columns: 1fr;
  }
  .protocol-url-row,
  .model-adder {
    align-items: stretch;
    flex-direction: column;
  }
  .protocol-url-row > span {
    width: auto;
  }
}
</style>
