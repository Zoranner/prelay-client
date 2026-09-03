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

export function useAgentSettings(options: AgentSettingsOptions) {
  const notifications = useNotification();
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
    groupEndpointModels(selectedEndpoint.value?.models ?? []).map((group) => ({
      value: group.name,
      label: group.displayName,
      catalogModel: catalogLanguageModel(group.catalogModel),
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

  function codexConnection() {
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
        models: groupEndpointModels(endpoint.models).map((group) => ({
          modelName: group.name,
          upstreamModel: group.mappings[0]?.model.upstream_model ?? group.name,
          catalogModel: catalogLanguageModel(group.catalogModel),
        })),
      };
    }
    return null;
  }

  function openCodeConnection() {
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
    await options.save({
      settings:
        client === "codexCli"
          ? { client, settings: codexSettingsPayload(draft.codexCli) }
          : client === "chatgpt"
            ? { client, settings: codexSettingsPayload(draft.chatgpt) }
            : { client, settings: openCodeSettingsPayload(draft.openCode) },
      connection: connection ? { client, connection } : null,
    });
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
