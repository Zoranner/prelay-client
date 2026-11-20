import type { AgentClientStatus } from "~/types/relay";

let clientStatusRequest: Promise<AgentClientStatus[]> | undefined;

export function useAgentClientStatus() {
  const { invokeLocalCommand } = useLocalCommand();
  const clientStatuses = useState<AgentClientStatus[]>(
    "agent-workspace-client-statuses",
    () => [],
  );
  const clientStatusesLoaded = useState(
    "agent-workspace-client-statuses-loaded",
    () => false,
  );
  const clientStatusesLoading = useState(
    "agent-workspace-client-statuses-loading",
    () => false,
  );

  async function refreshClientStatuses(): Promise<AgentClientStatus[]> {
    if (clientStatusRequest) return clientStatusRequest;

    clientStatusesLoaded.value = false;
    clientStatusesLoading.value = true;
    const request = (async () => {
      try {
        const statuses = await invokeLocalCommand<AgentClientStatus[]>(
          "agents_status",
          undefined,
          { notify: false, trackPending: false },
        );
        clientStatuses.value = statuses;
        return statuses;
      } catch {
        // Local detection failure keeps the last known installation snapshot.
        return clientStatuses.value;
      } finally {
        clientStatusesLoaded.value = true;
        clientStatusesLoading.value = false;
        clientStatusRequest = undefined;
      }
    })();
    clientStatusRequest = request;

    return request;
  }

  return {
    clientStatuses,
    clientStatusesLoaded,
    clientStatusesLoading,
    refreshClientStatuses,
  };
}
