import { useNotification } from "@stellar/ui";
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

export type ProviderOperationInput = {
  provider_type: string;
  base_url: string;
  api_key: string;
  protocol?: UpstreamProtocol;
  model?: string;
};

export type ProviderFormPayload = {
  id?: string;
  name: string;
  provider_type: string;
  base_url: string;
  api_key: string;
  capabilities: ProviderCapabilities;
  models: string[];
};

type ProviderFormOptions = {
  provider: () => Provider | null | undefined;
  discoverModels: (
    input: ProviderOperationInput,
  ) => Promise<ProviderOperationResult>;
  testProtocol: (
    input: ProviderOperationInput,
  ) => Promise<ProviderOperationResult>;
  onDirtyChange: (dirty: boolean) => void;
};

const allProtocols: UpstreamProtocol[] = [
  "openai",
  "responses",
  "anthropic",
  "images_generations",
];
const protocolOptions = allProtocols.map((protocol) => ({
  value: protocol,
  label: protocolLabel(protocol),
  tagVariant: protocolTagVariant(protocol),
}));
const providerTemplateOptions = PROVIDER_TEMPLATE_GROUPS.flatMap((group) =>
  group.options.map((option) => ({
    label: `${group.label} - ${option.label}`,
    value: option.value,
  })),
);

export function useProviderForm(options: ProviderFormOptions) {
  const providerTemplate = ref("custom");
  const name = ref("");
  const providerType = ref("openai_compatible");
  const baseUrl = ref("");
  const apiKey = ref("");
  const models = ref<string[]>([]);
  const modelDraft = ref("");
  const upstreamProtocols = ref<UpstreamProtocol[]>([]);
  const orderedUpstreamProtocols = computed(() =>
    allProtocols.filter((protocol) =>
      upstreamProtocols.value.includes(protocol),
    ),
  );
  const protocolBaseUrls = reactive<Record<UpstreamProtocol, string>>({
    responses: "",
    openai: "",
    anthropic: "",
    images_generations: "",
  });
  const showApiKey = ref(false);
  const showAddModel = ref(false);
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
  const defaultProviderTemplate = PROVIDER_TEMPLATE_GROUPS[0]?.options[0];
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

  function isProtocol(value: string): value is UpstreamProtocol {
    return allProtocols.includes(value as UpstreamProtocol);
  }

  function resetDraft(provider: Provider | null | undefined) {
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
    options.onDirtyChange(false);
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

  async function requestDiscovery() {
    if (!apiKey.value.trim()) {
      notifications.danger("请填写上游 API Key。", { title: "连接配置不完整" });
      return;
    }
    const result = await options.discoverModels(operationInput());
    if (!result.ok) {
      notifications.warning(
        `${result.error ?? "请检查连接信息和 API Key。"} 可手工添加模型后保存供应商。`,
        { title: "模型列表暂不可用" },
      );
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
    const result = await options.testProtocol(operationInput(protocol));
    const feedback = getProviderOperationFeedback(result);
    notifications.notify({
      type: feedback.success ? "success" : "danger",
      title: feedback.message,
      message: feedback.metrics ?? "",
    });
  }

  function submit(): ProviderFormPayload | null {
    if (!name.value.trim() || !baseUrl.value.trim()) {
      notifications.danger("请填写名称和 Base URL。", {
        title: "连接配置不完整",
      });
      return null;
    }
    const provider = options.provider();
    if (!provider && !apiKey.value.trim()) {
      notifications.danger("请填写上游 API Key。", { title: "连接配置不完整" });
      return null;
    }
    const payload = {
      ...(provider ? { id: provider.id } : {}),
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
    };
    apiKey.value = "";
    return payload;
  }

  watch(options.provider, resetDraft, { immediate: true });
  watch(serializeDraft, (draft) =>
    options.onDirtyChange(draft !== initialDraft),
  );

  return {
    addModel,
    allProtocols,
    apiKey,
    baseUrl,
    modelDraft,
    models,
    name,
    orderedUpstreamProtocols,
    protocolBaseUrls,
    protocolLabel,
    protocolOptions,
    providerTemplate,
    providerTemplateOptions,
    removeModel,
    requestDiscovery,
    requestProtocolTest,
    selectProviderTemplate,
    setModelPopover,
    showAddModel,
    showApiKey,
    submit,
    upstreamProtocols,
  };
}
