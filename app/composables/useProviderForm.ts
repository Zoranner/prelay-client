import { useNotification } from "@stellar/ui";
import type {
  CatalogProvider,
  Provider,
  ProviderCapabilities,
  UpstreamProtocol,
} from "~/stores/relay";
import {
  type ProviderTemplate,
  protocolLabel,
  providerModelOptions,
  providerTemplateForType,
  providerTemplates,
} from "~/utils/providerTemplates";
import {
  modelCatalogProviderModels,
  useModelCatalog,
} from "~/utils/modelCatalog";
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
  catalogProviders: () => CatalogProvider[];
  testProtocol: (
    input: ProviderOperationInput,
  ) => Promise<ProviderOperationResult>;
  onDirtyChange: (dirty: boolean) => void;
};

const allProtocols: UpstreamProtocol[] = [
  "openai",
  "responses",
  "images_generations",
];

export function useProviderForm(options: ProviderFormOptions) {
  const { status: catalogStatus } = useModelCatalog();
  const providerTemplateOptions = computed(() =>
    providerTemplates(options.catalogProviders()).map((option) => ({
      label: option.label,
      value: option.value,
    })),
  );
  const providerTemplate = ref("");
  const name = ref("");
  const providerType = ref("");
  const baseUrl = ref("");
  const apiKey = ref("");
  const languageModels = ref<string[]>([]);
  const imageGenerationModels = ref<string[]>([]);
  const models = computed(() => [
    ...languageModels.value,
    ...imageGenerationModels.value,
  ]);
  const languageModelOptions = computed(() =>
    providerModelOptions(languageModels.value).filter((option) =>
      Boolean(option.model && "reasoning_efforts" in option.model),
    ),
  );
  const imageGenerationModelOptions = computed(() =>
    providerModelOptions(imageGenerationModels.value).filter((option) =>
      Boolean(option.model && !("reasoning_efforts" in option.model)),
    ),
  );
  // 协议集合由供应商目录定义，表单只读展示；协议地址可以按协议覆盖，缺省用 Base URL。
  const currentTemplate = computed(() =>
    providerTemplates(options.catalogProviders()).find(
      (item) => item.providerType === providerType.value,
    ),
  );
  const orderedUpstreamProtocols = computed(() =>
    allProtocols.filter((protocol) =>
      currentTemplate.value?.protocols.includes(protocol),
    ),
  );
  const protocolBaseUrls = reactive<Record<UpstreamProtocol, string>>({
    responses: "",
    openai: "",
    images_generations: "",
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
  // 供应商自己保存的启用清单；目录条目只是可选的"菜单"。
  const enabledModels = ref<string[]>([]);
  const catalogModelIdOptions = computed(() => [
    ...languageModels.value,
    ...imageGenerationModels.value,
  ]);
  // 供应商清单里已经不在目录条目里的模型：目录里已经没有它们的类型信息，统一按已下架处理。
  const retiredModels = computed(() =>
    enabledModels.value.filter(
      (modelId) => !catalogModelIdOptions.value.includes(modelId),
    ),
  );
  const retiredProvider = computed(
    () => Boolean(options.provider()) && !currentTemplate.value,
  );
  const notifications = useNotification();
  let initialDraft = "";

  function isModelEnabled(modelId: string) {
    return enabledModels.value.includes(modelId);
  }

  function toggleModel(modelId: string) {
    enabledModels.value = isModelEnabled(modelId)
      ? enabledModels.value.filter((id) => id !== modelId)
      : [...enabledModels.value, modelId];
  }

  function removeRetiredModel(modelId: string) {
    enabledModels.value = enabledModels.value.filter((id) => id !== modelId);
  }

  function catalogModelIds(providerType: string, template?: ProviderTemplate) {
    const catalogModels = modelCatalogProviderModels(providerType);
    if (catalogStatus.value !== "ready") {
      return {
        language: template?.languageModels ?? [],
        image: template?.imageGenerationModels ?? [],
      };
    }
    return {
      language: catalogModels
        .filter((model) => "reasoning_efforts" in model)
        .map((model) => model.id),
      image: catalogModels
        .filter((model) => !("reasoning_efforts" in model))
        .map((model) => model.id),
    };
  }

  function serializeDraft() {
    return JSON.stringify({
      name: name.value,
      providerType: providerType.value,
      baseUrl: baseUrl.value,
      apiKey: apiKey.value,
      enabledModels: enabledModels.value,
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

  function resetDraft(provider: Provider | null | undefined) {
    const template = provider
      ? providerTemplateForType(
          provider.provider_type,
          options.catalogProviders(),
        )
      : providerTemplates(options.catalogProviders())[0];
    providerTemplate.value = template?.value ?? "";
    name.value = provider?.name ?? template?.label ?? "";
    providerType.value =
      provider?.provider_type ?? template?.providerType ?? "";
    baseUrl.value = provider?.base_url ?? template?.baseUrl ?? "";
    apiKey.value = provider?.api_key ?? "";
    const modelIds = catalogModelIds(providerType.value, template);
    languageModels.value = modelIds.language;
    imageGenerationModels.value = modelIds.image;
    for (const protocol of allProtocols) {
      protocolBaseUrls[protocol] =
        provider?.capabilities?.protocol_base_urls?.[protocol] ??
        template?.protocolBaseUrls[protocol] ??
        "";
    }
    preservedCapabilities.value = provider?.capabilities ?? {};
    enabledModels.value = provider
      ? [...provider.models]
      : [...modelIds.language, ...modelIds.image];
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
    const template = providerTemplates(options.catalogProviders()).find(
      (item) => item.value === providerTemplate.value,
    );
    if (!template) return;
    name.value = template.label;
    providerType.value = template.providerType;
    baseUrl.value = template.baseUrl;
    const modelIds = catalogModelIds(providerType.value, template);
    languageModels.value = modelIds.language;
    imageGenerationModels.value = modelIds.image;
    enabledModels.value = [...modelIds.language, ...modelIds.image];
    for (const protocol of allProtocols) {
      protocolBaseUrls[protocol] = template.protocolBaseUrls[protocol] ?? "";
    }
  }

  function operationInput(protocol?: UpstreamProtocol): ProviderOperationInput {
    return {
      provider_type: providerType.value,
      base_url:
        protocolBaseUrls[protocol ?? "openai"].trim() || baseUrl.value.trim(),
      api_key: apiKey.value.trim(),
      ...(protocol ? { protocol } : {}),
      ...(enabledModels.value[0]
        ? { model: enabledModels.value[0] }
        : models.value[0]
          ? { model: models.value[0] }
          : {}),
    };
  }

  async function requestProtocolTest(protocol: UpstreamProtocol) {
    if (!apiKey.value.trim()) {
      notifications.error("请填写上游 API Key。", { title: "连接配置不完整" });
      return;
    }
    const result = await options.testProtocol(operationInput(protocol));
    const feedback = getProviderOperationFeedback(result);
    notifications.notify({
      semantic: feedback.success ? "success" : "error",
      title: feedback.message,
      message: feedback.metrics ?? "",
    });
  }

  function submit(): ProviderFormPayload | null {
    if (!name.value.trim() || !baseUrl.value.trim()) {
      notifications.error("请填写名称和 Base URL。", {
        title: "连接配置不完整",
      });
      return null;
    }
    const provider = options.provider();
    if (!provider && !providerTemplate.value) {
      notifications.error("请选择目录中的供应商。", {
        title: "供应商配置不完整",
      });
      return null;
    }
    if (!provider && !apiKey.value.trim()) {
      notifications.error("请填写上游 API Key。", { title: "连接配置不完整" });
      return null;
    }
    if (catalogStatus.value !== "ready") {
      notifications.error("模型目录尚未加载完成，请稍后重试。", {
        title: "无法保存供应商配置",
      });
      return null;
    }
    if (retiredProvider.value) {
      notifications.error("该供应商的目录条目已下架，只能删除。", {
        title: "无法保存供应商配置",
      });
      return null;
    }
    if (retiredModels.value.length) {
      notifications.error(
        `请先移除目录里已下架的模型：${retiredModels.value.join("、")}`,
        { title: "无法保存供应商配置" },
      );
      return null;
    }
    const payload = {
      ...(provider ? { id: provider.id } : {}),
      name: name.value.trim(),
      provider_type: providerType.value,
      base_url: baseUrl.value.trim(),
      api_key: apiKey.value,
      models: [...enabledModels.value],
      capabilities: {
        ...preservedCapabilities.value,
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
    };
    apiKey.value = "";
    return payload;
  }

  watch(options.provider, resetDraft, { immediate: true });
  watch(serializeDraft, (draft) =>
    options.onDirtyChange(draft !== initialDraft),
  );

  return {
    allProtocols,
    apiKey,
    baseUrl,
    enabledModels,
    isModelEnabled,
    removeRetiredModel,
    retiredModels,
    retiredProvider,
    toggleModel,
    languageModels,
    imageGenerationModels,
    imageGenerationModelOptions,
    languageModelOptions,
    models,
    name,
    orderedUpstreamProtocols,
    protocolBaseUrls,
    protocolLabel,
    providerTemplate,
    providerTemplateOptions,
    requestProtocolTest,
    selectProviderTemplate,
    submit,
  };
}
