import { useNotification } from "@stellar/ui";
import type {
  AgentClient,
  AgentSettings,
  BootstrapState,
  CatalogLanguageModelResponse,
  RelayEndpoint,
} from "~/types/relay";
import { clientSupportsSettings } from "~/utils/agent/client";
import {
  catalogLanguageModel,
  codexConnectionFor,
  managementBaseUrl,
  normalizeBaseUrl,
  type CodexConnectionDraft,
} from "~/utils/agent/connections";
import { groupEndpointModels } from "~/utils/model/endpoints";
import { modelCatalogEntry, useModelCatalog } from "~/stores/modelCatalog";
import {
  codexSettingsPayload,
  copyAgentClientSettings,
  createAgentConfiguration,
  type CodexSettingsDraft,
  type AgentSettingsSaveRequest,
} from "~/utils/agent/settings";
import { normalizeReasoningEffort } from "~/utils/model/reasoning";

type AgentSettingsOptions = {
  activeClient: Readonly<Ref<AgentClient>>;
  bootstrap: Readonly<Ref<BootstrapState | null>>;
  endpoints: Readonly<Ref<RelayEndpoint[]>>;
  settings: Readonly<Ref<Partial<Record<AgentClient, AgentSettings>>>>;
  reloadSettings: (client: AgentClient) => Promise<void>;
  save: (request: AgentSettingsSaveRequest) => Promise<unknown>;
};

export function validatePrelayModelSelection(
  status: ReturnType<typeof useModelCatalog>["status"]["value"],
  selectedModel: string,
  endpointModelIds: string[],
  reasoningEffort = "",
) {
  if (status !== "ready") return "模型目录尚未加载完成，请稍后重试。";
  const model = modelCatalogEntry(selectedModel);
  if (
    !selectedModel ||
    !endpointModelIds.includes(selectedModel) ||
    !model ||
    !("reasoning_efforts" in model)
  ) {
    return "接入点包含目录外模型，无法保存。";
  }
  if (
    reasoningEffort.trim() !== "" &&
    !model.reasoning_efforts?.includes(reasoningEffort)
  ) {
    return "当前模型不支持所选推理强度，无法保存。";
  }
  return null;
}

export async function saveWithAgentValidation(options: {
  kind: "prelay" | null;
  status: ReturnType<typeof useModelCatalog>["status"]["value"];
  selectedModel: string;
  endpointModelIds: string[];
  reasoningEffort?: string;
  save: () => Promise<void>;
}) {
  if (options.kind === "prelay") {
    const validationError = validatePrelayModelSelection(
      options.status,
      options.selectedModel,
      options.endpointModelIds,
      options.reasoningEffort,
    );
    if (
      validationError ||
      options.endpointModelIds.some(
        (modelId) => !catalogLanguageModel(modelCatalogEntry(modelId)),
      )
    ) {
      return validationError ?? "接入点包含目录外模型，无法保存。";
    }
  }
  await options.save();
  return null;
}

export function useAgentSettings(options: AgentSettingsOptions) {
  const notifications = useNotification();
  const { status: catalogStatus } = useModelCatalog();
  const configuration = reactive(createAgentConfiguration());
  const draft = reactive(createAgentConfiguration());
  const showSettings = ref(false);
  const activeSettings = computed(() => draft[options.activeClient.value]);
  const selectedEndpoint = computed(() =>
    options.endpoints.value.find(
      (endpoint) => endpoint.id === activeSettings.value.endpoint,
    ),
  );
  function endpointOptionsForClient() {
    const endpointOptions = options.endpoints.value.map((endpoint) => ({
      value: endpoint.id,
      label: endpoint.name,
      description: `${groupEndpointModels(endpoint.models).length} 个模型`,
    }));
    return endpointOptions;
  }
  const endpointOptions = computed(() => endpointOptionsForClient());
  const modelOptions = computed(() =>
    groupEndpointModels(selectedEndpoint.value?.models ?? [])
      .map((group) => catalogLanguageModel(group.catalogModel))
      .filter((model): model is CatalogLanguageModelResponse => Boolean(model))
      .map((model) => ({
        value: model.id,
        label: model.display_name,
        catalogModel: model,
      })),
  );
  const dirty = computed(
    () => JSON.stringify(draft) !== JSON.stringify(configuration),
  );

  function normalizeCodexDraftReasoning(target: CodexSettingsDraft) {
    const selected = modelOptions.value.find(
      (option) => option.value === target.model,
    )?.catalogModel;
    if (selected && "reasoning_efforts" in selected) {
      target.reasoningEffort = normalizeReasoningEffort(
        target.reasoningEffort,
        selected,
      );
    }
  }

  watch(
    [
      () => draft.codexCli.endpoint,
      () => draft.codexCli.model,
      () => draft.chatgpt.endpoint,
      () => draft.chatgpt.model,
      modelOptions,
    ],
    () => {
      normalizeCodexDraftReasoning(draft.codexCli);
      normalizeCodexDraftReasoning(draft.chatgpt);
    },
  );

  function open(installed: boolean) {
    const client = options.activeClient.value;
    if (!installed || !clientSupportsSettings(client)) return;
    copyAgentClientSettings(configuration, draft, client);
    showSettings.value = true;
  }

  function close() {
    showSettings.value = false;
  }

  function discard() {
    copyAgentClientSettings(configuration, draft, "codexCli");
    copyAgentClientSettings(configuration, draft, "chatgpt");
    close();
  }

  function codexConnection(): CodexConnectionDraft | null {
    const endpoint = selectedEndpoint.value;
    const relayUrl = options.bootstrap.value?.relay_url;
    if (!endpoint || !relayUrl) return null;
    return codexConnectionFor(endpoint, relayUrl);
  }

  async function save() {
    const client = options.activeClient.value;
    const connection = codexConnection();
    const request: AgentSettingsSaveRequest = {
      settings:
        client === "codexCli"
          ? { client, settings: codexSettingsPayload(draft.codexCli) }
          : { client, settings: codexSettingsPayload(draft.chatgpt) },
      connection: connection ? { client, connection } : null,
    };
    let savedByValidation = false;
    if (connection?.kind === "prelay") {
      const modelIds = (
        connection as Extract<CodexConnectionDraft, { kind: "prelay" }>
      ).models.map((model) => model.id);
      const selectedModel =
        client === "codexCli" ? draft.codexCli.model : draft.chatgpt.model;
      const validationError = await saveWithAgentValidation({
        kind: "prelay",
        status: catalogStatus.value,
        selectedModel,
        endpointModelIds: modelIds,
        reasoningEffort:
          client === "codexCli"
            ? draft.codexCli.reasoningEffort
            : draft.chatgpt.reasoningEffort,
        save: async () => {
          await options.save(request);
          savedByValidation = true;
        },
      });
      if (validationError) {
        notifications.error(validationError, {
          title: "无法保存智能体设置",
        });
        return false;
      }
    }
    if (!savedByValidation) await options.save(request);
    copyAgentClientSettings(draft, configuration, client);
    close();
    notifications.success("设置已保存");
    void options.reloadSettings(client);
  }

  function hydrate(value: AgentSettings) {
    const managementUrl = options.bootstrap.value?.relay_url
      ? managementBaseUrl(options.bootstrap.value.relay_url)
      : null;
    const { endpointName, baseUrl, ...settings } = value.settings;
    const target = configuration[value.client];
    Object.assign(target, settings);
    Object.assign(target.features, value.settings.features);
    const endpoint =
      managementUrl && baseUrl && normalizeBaseUrl(baseUrl) === managementUrl
        ? options.endpoints.value.find((item) => item.name === endpointName)
        : undefined;
    target.endpoint = endpoint?.id ?? "";
    copyAgentClientSettings(configuration, draft, value.client);
  }

  watch(
    [
      () => options.settings.value.codexCli,
      () => options.settings.value.chatgpt,
      () => options.endpoints.value,
      () => options.bootstrap.value?.relay_url,
    ],
    ([codexCli, chatgpt]) => {
      if (codexCli) hydrate(codexCli);
      if (chatgpt) hydrate(chatgpt);
    },
    { immediate: true },
  );

  return {
    configuration,
    dirty,
    discard,
    draft,
    endpointOptions,
    modelOptions,
    open,
    save,
    showSettings,
    close,
  };
}
