import type {
  AgentClient,
  AgentClientVersion,
  AgentItemsSnapshot,
  AgentSettings,
} from "~/stores/relay";

const clients: AgentClient[] = ["codex", "claudeCode"];

function emptySnapshot(): AgentItemsSnapshot {
  return { clients: [] };
}

function emptyClientFlags() {
  return {
    codex: false,
    claudeCode: false,
  } satisfies Record<AgentClient, boolean>;
}

let loadPromise: Promise<void> | undefined;
let loadGeneration = 0;

export function useAgentWorkspace() {
  const { invokeLocalCommand } = useLocalCommand();
  const snapshot = useState<AgentItemsSnapshot>(
    "agent-workspace-snapshot",
    emptySnapshot,
  );
  const loaded = useState("agent-workspace-loaded", () => false);
  const loading = useState("agent-workspace-loading", () => false);
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

  function resetSettings(detectedClients: AgentClient[]) {
    const detected = new Set(detectedClients);
    for (const client of clients) {
      settingsLoaded.value[client] = false;
      settingsLoading.value[client] = detected.has(client);
    }
  }

  async function loadVersions(
    detectedClients: AgentClient[],
    generation: number,
  ) {
    try {
      const versions = await invokeLocalCommand<AgentClientVersion[]>(
        "agents_versions",
        { clients: detectedClients },
        { notify: false, trackPending: false },
      );
      const versionsByClient = new Map(
        versions.map(({ client, version }) => [client, version]),
      );
      if (generation !== loadGeneration) return;
      snapshot.value = {
        clients: snapshot.value.clients.map((client) => ({
          ...client,
          version: versionsByClient.get(client.client) ?? client.version,
        })),
      };
    } catch {
      // Version detection is an optional background enhancement.
    }
  }

  async function loadSettings(
    client: AgentClient,
    force = false,
    generation = loadGeneration,
  ) {
    if (!force && (settingsLoaded.value[client] || settingsLoading.value[client])) {
      return;
    }

    settingsLoading.value[client] = true;
    try {
      const value = await invokeLocalCommand<AgentSettings>(
        "agent_settings_get",
        { client },
        { notify: false, trackPending: false },
      );
      if (generation !== loadGeneration) return;
      settings.value = { ...settings.value, [client]: value };
      settingsLoaded.value[client] = true;
    } catch {
      // The page-level loading state represents unavailable local settings.
    } finally {
      if (generation === loadGeneration) {
        settingsLoading.value[client] = false;
      }
    }
  }

  async function load(force = false) {
    if (loaded.value && !force) return;
    if (loadPromise) return loadPromise;

    loading.value = true;
    const generation = ++loadGeneration;
    loadPromise = (async () => {
      try {
        snapshot.value = await invokeLocalCommand<AgentItemsSnapshot>(
          "agents_list",
          undefined,
          { notify: false, trackPending: false },
        );
        loaded.value = true;
        const detectedClients = snapshot.value.clients.map(({ client }) => client);
        resetSettings(detectedClients);
        void loadVersions(detectedClients, generation);
        void Promise.all(
          detectedClients.map((client) => loadSettings(client, true, generation)),
        );
      } finally {
        loading.value = false;
        loadPromise = undefined;
      }
    })();

    return loadPromise;
  }

  async function refresh() {
    await load(true);
  }

  async function reloadSettings(client: AgentClient) {
    await loadSettings(client, true);
  }

  return {
    snapshot,
    loaded,
    loading,
    settings,
    settingsLoading,
    settingsLoaded,
    load,
    refresh,
    reloadSettings,
  };
}
