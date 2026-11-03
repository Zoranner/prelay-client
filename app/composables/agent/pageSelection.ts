import type {
  AgentClient,
  AgentClientItems,
  AgentClientStatus,
  AgentItemKind,
} from "~/types/relay";
import {
  agentClientDefinitions,
  agentSectionOptions,
  sortAgentClients,
  type AgentSection,
  type AgentSectionOption,
  type AgentWorkspace,
} from "~/utils/agent/client";

type AgentPageWorkspace = {
  clientStatuses: Ref<AgentClientStatus[]>;
  clientStatusesLoaded: Ref<boolean>;
  clientItems: Ref<Partial<Record<AgentClient, AgentClientItems>>>;
  itemsLoading: Ref<Record<AgentClient, boolean>>;
  settingsLoading: Ref<Record<AgentClient, boolean>>;
  refreshClient: (client: AgentClient) => Promise<void>;
};

export function useAgentPageSelection(workspace: AgentPageWorkspace) {
  const {
    clientItems,
    clientStatuses,
    clientStatusesLoaded,
    itemsLoading,
    settingsLoading,
  } = workspace;
  const agentClients = computed(() =>
    sortAgentClients(
      agentClientDefinitions.map((definition) => {
        const status = clientStatuses.value.find(
          ({ client }) => client === definition.client,
        );
        return {
          ...definition,
          installed: clientStatusesLoaded.value && Boolean(status?.installed),
          version: clientStatusesLoaded.value ? (status?.version ?? "-") : "-",
        };
      }),
    ),
  );
  const initialClient = agentClients.value[0]?.client ?? "chatgpt";
  const activeWorkspace = ref<AgentWorkspace>(initialClient);
  const lastActiveClient = ref<AgentClient>(initialClient);
  const activeClient = computed<AgentClient>(() =>
    activeWorkspace.value === "extensions"
      ? lastActiveClient.value
      : activeWorkspace.value,
  );
  const activeSection = ref<AgentSection>("rules");
  const availableSectionOptions = computed<AgentSectionOption[]>(() => {
    const definition = agentClientDefinitions.find(
      (candidate) => candidate.client === activeClient.value,
    );
    return agentSectionOptions.filter(({ value }) =>
      definition?.sections.includes(value),
    );
  });
  const activeClientDetected = computed(() =>
    isClientInstalled(activeClient.value),
  );
  const activeItems = computed(
    () => clientItems.value[activeClient.value]?.items ?? [],
  );
  const activeKind = computed<AgentItemKind | null>(() =>
    activeSection.value === "rules" ? null : activeSection.value,
  );
  const sectionItems = computed(() =>
    activeKind.value
      ? activeItems.value.filter((item) => item.kind === activeKind.value)
      : [],
  );

  function isClientInstalled(client: AgentClient) {
    return clientStatuses.value.some(
      (status) => status.client === client && status.installed,
    );
  }

  function isAgentContentLoading(client: AgentClient) {
    return itemsLoading.value[client] || settingsLoading.value[client];
  }

  function activateClient(client: AgentClient) {
    lastActiveClient.value = client;
    activeWorkspace.value = client;
  }

  function activateExtensions() {
    activeWorkspace.value = "extensions";
  }

  watch(
    availableSectionOptions,
    (options) => {
      if (!options.some(({ value }) => value === activeSection.value)) {
        activeSection.value = options[0]?.value ?? "rules";
      }
    },
    { immediate: true },
  );

  watch(
    clientStatuses,
    () => {
      if (
        activeWorkspace.value !== "extensions" &&
        !activeClientDetected.value
      ) {
        const fallback = agentClients.value[0]?.client ?? "chatgpt";
        lastActiveClient.value = fallback;
        activeWorkspace.value = fallback;
      }
    },
    { immediate: true },
  );

  watch(
    [clientStatusesLoaded, activeWorkspace],
    () => {
      if (
        activeWorkspace.value !== "extensions" &&
        clientStatusesLoaded.value &&
        activeClientDetected.value
      ) {
        void workspace.refreshClient(activeClient.value);
      }
    },
    { immediate: true },
  );

  return {
    activeClient,
    activeClientDetected,
    activeSection,
    activeWorkspace,
    agentClients,
    availableSectionOptions,
    activateClient,
    activateExtensions,
    isAgentContentLoading,
    sectionItems,
  };
}
