import { useNotification } from "@stellar/ui";
import type {
  CatalogProvider,
  Provider,
  ProviderCapabilities,
  UpstreamProtocol,
} from "~/stores/relay";
import {
  protocolLabel,
  providerModelOptions,
  providerTemplateForType,
  providerTemplates,
} from "~/utils/providerTemplates";
import {
  modelCatalogEntry,
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
  "anthropic",
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
    const savedModelIds =
      provider?.models.map((model) => model.model_name) ?? [];
    const imageModelIds = new Set(template?.imageGenerationModels ?? []);
    const catalogModelIds = new Set(
      modelCatalogProviderModels(providerType.value).map((model) => model.id),
    );
    const catalogReady = catalogStatus.value === "ready";
    languageModels.value = provider
      ? savedModelIds.filter(
          (id) =>
            !imageModelIds.has(id) &&
            (!catalogReady || catalogModelIds.has(id)),
        )
      : (template?.languageModels ?? []).filter(
          (id) =>
            !catalogReady ||
            (catalogModelIds.has(id) && Boolean(modelCatalogEntry(id))),
        );
    imageGenerationModels.value = provider
      ? savedModelIds.filter(
          (id) =>
            imageModelIds.has(id) && (!catalogReady || catalogModelIds.has(id)),
        )
      : (template?.imageGenerationModels ?? []).filter(
          (id) =>
            !catalogReady ||
            (catalogModelIds.has(id) && Boolean(modelCatalogEntry(id))),
        );
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
    const template = providerTemplates(options.catalogProviders()).find(
      (item) => item.value === providerTemplate.value,
    );
    if (!template) return;
    name.value = template.label;
    providerType.value = template.providerType;
    const catalogModelIds = new Set(
      modelCatalogProviderModels(providerType.value).map((model) => model.id),
    );
    baseUrl.value = template.baseUrl;
    const catalogReady = catalogStatus.value === "ready";
    languageModels.value = [...template.languageModels].filter(
      (id) =>
        !catalogReady ||
        (catalogModelIds.has(id) && Boolean(modelCatalogEntry(id))),
    );
    imageGenerationModels.value = [...template.imageGenerationModels].filter(
      (id) =>
        !catalogReady ||
        (catalogModelIds.has(id) && Boolean(modelCatalogEntry(id))),
    );
    upstreamProtocols.value = [...template.protocols];
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
      ...(models.value[0] ? { model: models.value[0] } : {}),
    };
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
    if (!provider && !providerTemplate.value) {
      notifications.danger("请选择目录中的供应商。", {
        title: "供应商配置不完整",
      });
      return null;
    }
    if (!provider && !apiKey.value.trim()) {
      notifications.danger("请填写上游 API Key。", { title: "连接配置不完整" });
      return null;
    }
    if (catalogStatus.value !== "ready") {
      notifications.danger("模型目录尚未加载完成，请稍后重试。", {
        title: "无法保存供应商配置",
      });
      return null;
    }
    if (models.value.some((id) => !modelCatalogEntry(id))) {
      notifications.danger("请选择目录中的模型。", {
        title: "供应商配置不完整",
      });
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
    allProtocols,
    apiKey,
    baseUrl,
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
    upstreamProtocols,
  };
}
