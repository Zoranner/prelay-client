import { useNotification } from "@stellar/ui";
import type {
  AgentClient,
  AgentSettings,
  BootstrapState,
  CatalogLanguageModelResponse,
  CatalogModelResponse,
  RelayEndpoint,
} from "~/stores/relay";
import { clientSupportsSettings } from "~/utils/agentClient";
import { groupEndpointModels } from "~/utils/endpointModels";
import { modelCatalogEntry, useModelCatalog } from "~/utils/modelCatalog";
import {
  codexSettingsPayload,
  copyAgentClientSettings,
  createAgentConfiguration,
  openCodeSettingsPayload,
  type AgentSettingsSaveRequest,
} from "~/utils/agentSettings";

export const customEndpointValue = "__custom__";

type AgentSettingsOptions = {
  activeClient: Readonly<Ref<AgentClient>>;
  bootstrap: Readonly<Ref<BootstrapState | null>>;
  endpoints: Readonly<Ref<RelayEndpoint[]>>;
  settings: Readonly<Ref<Partial<Record<AgentClient, AgentSettings>>>>;
  reloadSettings: (client: AgentClient) => Promise<void>;
  save: (request: AgentSettingsSaveRequest) => Promise<unknown>;
};

type CodexConnectionDraft =
  | {
      kind: "prelay";
      endpointId: string;
      endpointName: string;
      endpointToken: string;
      relayUrl: string;
      models: CatalogLanguageModelResponse[];
    }
  | {
      kind: "custom";
      baseUrl: string;
      token: string;
    };

function normalizeBaseUrl(url: string) {
  return url.trim().replace(/\/+$/, "");
}

function managementBaseUrl(relayUrl: string) {
  const normalized = normalizeBaseUrl(relayUrl);
  return normalized.endsWith("/v1") ? normalized : `${normalized}/v1`;
}

function catalogLanguageModel(
  model: CatalogModelResponse | undefined,
): CatalogLanguageModelResponse | undefined {
  return model && "reasoning_efforts" in model ? model : undefined;
}

export function validatePrelayModelSelection(
  status: ReturnType<typeof useModelCatalog>["status"]["value"],
  selectedModel: string,
  endpointModelIds: string[],
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
  return null;
}

export async function saveWithAgentValidation(options: {
  kind: "prelay" | "custom" | null;
  status: ReturnType<typeof useModelCatalog>["status"]["value"];
  selectedModel: string;
  endpointModelIds: string[];
  save: () => Promise<void>;
}) {
  if (options.kind === "prelay") {
    const validationError = validatePrelayModelSelection(
      options.status,
      options.selectedModel,
      options.endpointModelIds,
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
  const endpointOptions = computed(() => [
    ...options.endpoints.value.map((endpoint) => ({
      value: endpoint.id,
      label: endpoint.name,
      description: `${groupEndpointModels(endpoint.models).length} 个模型`,
    })),
    { value: customEndpointValue, label: "自定义" },
  ]);
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
  const isCustomCodexEndpoint = computed(
    () => activeSettings.value.endpoint === customEndpointValue,
  );
  const dirty = computed(
    () => JSON.stringify(draft) !== JSON.stringify(configuration),
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
    copyAgentClientSettings(configuration, draft, "openCode");
    close();
  }

  function codexConnection(): CodexConnectionDraft | null {
    const codex =
      options.activeClient.value === "codexCli"
        ? draft.codexCli
        : draft.chatgpt;
    const customBaseUrl = codex.customBaseUrl.trim();
    if (isCustomCodexEndpoint.value && customBaseUrl) {
      return {
        kind: "custom",
        baseUrl: customBaseUrl,
        token: codex.customToken,
      };
    }
    const endpoint = selectedEndpoint.value;
    if (endpoint && options.bootstrap.value?.relay_url) {
      return {
        kind: "prelay",
        endpointId: endpoint.id,
        endpointName: endpoint.name,
        endpointToken: endpoint.token,
        relayUrl: options.bootstrap.value.relay_url,
        models: groupEndpointModels(endpoint.models)
          .map((group) => catalogLanguageModel(group.catalogModel))
          .filter(
            (model): model is CatalogLanguageModelResponse => Boolean(model),
          ),
      };
    }
    return null;
  }

  function openCodeConnection(): {
    kind: "prelay";
    endpointToken: string;
    relayUrl: string;
  } | null {
    const endpoint = selectedEndpoint.value;
    if (endpoint && options.bootstrap.value?.relay_url) {
      return {
        kind: "prelay",
        endpointToken: endpoint.token,
        relayUrl: options.bootstrap.value.relay_url,
      };
    }
    return null;
  }

  async function save() {
    const client = options.activeClient.value;
    const connection =
      client === "codexCli" || client === "chatgpt"
        ? codexConnection()
        : openCodeConnection();
    const request: AgentSettingsSaveRequest = {
      settings:
        client === "codexCli"
          ? { client, settings: codexSettingsPayload(draft.codexCli) }
          : client === "chatgpt"
            ? { client, settings: codexSettingsPayload(draft.chatgpt) }
            : { client, settings: openCodeSettingsPayload(draft.openCode) },
      connection: connection ? { client, connection } : null,
    };
    let savedByValidation = false;
    if (connection?.kind === "prelay") {
      const modelIds =
        client === "openCode"
          ? [draft.openCode.model]
          : (connection as Extract<
                CodexConnectionDraft,
                { kind: "prelay" }
              >).models.map((model) => model.id);
      const selectedModel =
        client === "codexCli"
          ? draft.codexCli.model
          : client === "chatgpt"
            ? draft.chatgpt.model
            : draft.openCode.model;
      const validationError = await saveWithAgentValidation({
        kind: "prelay",
        status: catalogStatus.value,
        selectedModel,
        endpointModelIds: modelIds,
        save: async () => {
          await options.save(request);
          savedByValidation = true;
        },
      });
      if (validationError) {
        notifications.danger(validationError, {
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
    if (value.client === "codexCli" || value.client === "chatgpt") {
      const { endpointName, baseUrl, customToken, ...settings } =
        value.settings;
      const target = configuration[value.client];
      Object.assign(target, settings);
      Object.assign(target.features, value.settings.features);
      target.customBaseUrl = baseUrl ?? "";
      const endpoint =
        managementUrl && baseUrl && normalizeBaseUrl(baseUrl) === managementUrl
          ? options.endpoints.value.find((item) => item.name === endpointName)
          : undefined;
      target.endpoint = endpoint?.id ?? customEndpointValue;
      target.customToken = endpoint ? "" : (customToken ?? "");
      copyAgentClientSettings(configuration, draft, value.client);
      return;
    }

    const { baseUrl, endpointToken, ...settings } = value.settings;
    Object.assign(configuration.openCode, settings);
    const endpoint =
      managementUrl && baseUrl && normalizeBaseUrl(baseUrl) === managementUrl
        ? options.endpoints.value.find((item) => item.token === endpointToken)
        : undefined;
    configuration.openCode.endpoint = endpoint?.id ?? customEndpointValue;
    copyAgentClientSettings(configuration, draft, "openCode");
  }

  watch(
    [
      () => options.settings.value.codexCli,
      () => options.settings.value.chatgpt,
      () => options.settings.value.openCode,
      () => options.endpoints.value,
      () => options.bootstrap.value?.relay_url,
    ],
    ([codexCli, chatgpt, openCode]) => {
      if (codexCli) hydrate(codexCli);
      if (chatgpt) hydrate(chatgpt);
      if (openCode) hydrate(openCode);
    },
    { immediate: true },
  );

  return {
    configuration,
    customEndpointValue,
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
