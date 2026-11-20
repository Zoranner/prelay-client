import type {
  AgentClient,
  AgentClientItems,
  AgentSettings,
} from "~/types/relay";
import { clientSupportsSettings } from "~/utils/agent/client";
import { useAgentClientStatus } from "~/composables/agent/clientStatus";
import { waitForAgentSync } from "~/composables/agent/sync";

function emptyClientFlags() {
  return {
    codexCli: false,
    chatgpt: false,
  } satisfies Record<AgentClient, boolean>;
}

const clientContentRequests: Partial<Record<AgentClient, Promise<void>>> = {};

export function useAgentWorkspace() {
  const { invokeLocalCommand } = useLocalCommand();
  const {
    clientStatuses,
    clientStatusesLoaded,
    clientStatusesLoading,
    refreshClientStatuses,
  } = useAgentClientStatus();
  const clientItems = useState<Partial<Record<AgentClient, AgentClientItems>>>(
    "agent-workspace-client-items",
    () => ({}),
  );
  const itemsLoading = useState<Record<AgentClient, boolean>>(
    "agent-workspace-items-loading",
    emptyClientFlags,
  );
  const settings = useState<Partial<Record<AgentClient, AgentSettings>>>(
    "agent-workspace-settings",
    () => ({}),
  );
  const settingsLoading = useState<Record<AgentClient, boolean>>(
    "agent-workspace-settings-loading",
    emptyClientFlags,
  );
  const settingsLoaded = useState<Record<AgentClient, boolean>>(
    "agent-workspace-settings-loaded",
    emptyClientFlags,
  );

  async function loadClientItems(client: AgentClient, force = false) {
    if (!force && (clientItems.value[client] || itemsLoading.value[client])) {
      return;
    }
    if (clientContentRequests[client]) return clientContentRequests[client];

    itemsLoading.value[client] = true;
    clientContentRequests[client] = (async () => {
      try {
        const items = await invokeLocalCommand<AgentClientItems>(
          "agent_items_get",
          { client },
          { notify: false, trackPending: false },
        );
        clientItems.value = { ...clientItems.value, [client]: items };
      } catch {
        // The page represents unavailable local content with an empty result.
      } finally {
        itemsLoading.value[client] = false;
        clientContentRequests[client] = undefined;
      }
    })();

    return clientContentRequests[client];
  }

  async function loadSettings(client: AgentClient, force = false) {
    if (!clientSupportsSettings(client)) return;
    if (
      !force &&
      (settingsLoaded.value[client] || settingsLoading.value[client])
    ) {
      return;
    }

    settingsLoading.value[client] = true;
    try {
      // 接入点改动后的对账先落盘，避免读到旧连接信息再原样写回。
      await waitForAgentSync();
      const value = await invokeLocalCommand<AgentSettings>(
        "agent_settings_get",
        { client },
        { notify: false, trackPending: false },
      );
      settings.value = { ...settings.value, [client]: value };
      settingsLoaded.value[client] = true;
    } catch {
      // The page-level loading state represents unavailable local settings.
    } finally {
      settingsLoading.value[client] = false;
    }
  }

  async function refreshClient(client: AgentClient) {
    await Promise.all([
      loadClientItems(client, true),
      loadSettings(client, true),
    ]);
  }

  async function reloadSettings(client: AgentClient) {
    await loadSettings(client, true);
  }

  return {
    clientStatuses,
    clientStatusesLoaded,
    clientStatusesLoading,
    clientItems,
    itemsLoading,
    settings,
    settingsLoading,
    settingsLoaded,
    refreshClientStatuses,
    refreshClient,
    reloadSettings,
  };
}
