import { useNotification } from "@stellar/ui";
import type {
  AgentClient,
  AgentClientStatus,
  AgentSettings,
  CodexSettings,
  OpenCodeSettings,
  RelayEndpoint,
} from "~/stores/relay";
import {
  codexConnectionFor,
  openCodeConnectionFor,
} from "~/utils/agentConnections";
import { agentClientDefinitions } from "~/utils/agentClient";
import { modelCatalogEntry, useModelCatalog } from "~/utils/modelCatalog";
import { normalizeReasoningEffort } from "~/utils/modelReasoning";
import {
  agentHost,
  agentModelFallback,
  linkedEndpoint,
  type AgentEndpointLinks,
} from "~/utils/agentSync";
import { useLocalCommand } from "~/composables/useLocalCommand";
import { useRelayCommand } from "~/composables/useRelayCommand";
import { useRelayStore } from "~/stores/relay";

export type AgentRewriteStatus =
  "synced" | "unchanged" | "missing_endpoint" | "not_linked" | "skipped";

export type AgentSyncOutcome = {
  client: AgentClient;
  status: AgentRewriteStatus;
  fallback?: { from: string; to: string };
};

/** Codex CLI 与 ChatGPT 共用一份宿主配置，所以按宿主对账，只写一次。 */
const SYNC_HOSTS: Array<{ host: "codex" | "opencode"; client: AgentClient }> = [
  { host: "codex", client: "codexCli" },
  { host: "opencode", client: "openCode" },
];

let pendingSync: Promise<AgentSyncOutcome[]> | null = null;

/** 智能体页读取设置前先等对账落盘，避免读到旧副本再写回去。 */
export function waitForAgentSync() {
  return pendingSync ?? Promise.resolve([]);
}

function clientLabel(client: AgentClient) {
  return (
    agentClientDefinitions.find((definition) => definition.client === client)
      ?.label ?? client
  );
}

export function useAgentSync() {
  const notifications = useNotification();
  const { invokeCommand } = useRelayCommand();
  const { invokeLocalCommand } = useLocalCommand();
  const { status: catalogStatus } = useModelCatalog();
  const { bootstrap } = useRelayStore();

  function notify(outcome: AgentSyncOutcome) {
    const label = clientLabel(outcome.client);
    if (outcome.status === "synced") {
      notifications.success(
        outcome.fallback
          ? `默认模型已从 ${outcome.fallback.from} 回退为 ${outcome.fallback.to}，其余设置保持不变。`
          : "连接信息与模型清单已同步到当前接入点。",
        { title: `${label} 已同步` },
      );
    }
    if (outcome.status === "missing_endpoint") {
      notifications.warning("引用的接入点已不存在，请在设置里重新选择。", {
        title: `${label} 需要处理`,
      });
    }
  }

  async function syncHost(
    target: { host: "codex" | "opencode"; client: AgentClient },
    endpoints: RelayEndpoint[],
    links: AgentEndpointLinks,
    statuses: AgentClientStatus[],
    relayUrl: string,
    matches?: (endpointId: string) => boolean,
  ): Promise<AgentSyncOutcome> {
    const client = target.client;
    if (!statuses.find((status) => status.client === client)?.installed) {
      return { client, status: "skipped" };
    }
    const settings = await invokeLocalCommand<AgentSettings>(
      "agent_settings_get",
      { client },
      { notify: false, trackPending: false },
    );
    const endpoint = linkedEndpoint({
      client,
      settings: settings.settings,
      links,
      endpoints,
      relayUrl,
    });
    if (!endpoint) {
      return {
        client,
        status: links[agentHost(client)] ? "missing_endpoint" : "not_linked",
      };
    }
    if (matches && !matches(endpoint.id)) {
      return { client, status: "skipped" };
    }
    const selection = agentModelFallback(
      endpoint,
      (settings.settings as { model?: string }).model,
    );
    if (!selection) return { client, status: "skipped" };

    const request =
      client === "openCode"
        ? buildOpenCodeRequest(
            settings.settings,
            selection.model,
            endpoint,
            relayUrl,
            client,
          )
        : buildCodexRequest(
            settings.settings,
            selection.model,
            endpoint,
            relayUrl,
            client,
          );
    const changed = await invokeLocalCommand<boolean>(
      "agent_settings_save",
      request,
      { notify: true, trackPending: false },
    );
    return {
      client,
      status: changed ? "synced" : "unchanged",
      ...(selection.fallback ? { fallback: selection.fallback } : {}),
    };
  }

  function buildCodexRequest(
    settings: AgentSettings["settings"],
    model: string,
    endpoint: RelayEndpoint,
    relayUrl: string,
    client: AgentClient,
  ) {
    const { endpointName, baseUrl, ...rest } = settings as CodexSettings;
    const catalogModel = modelCatalogEntry(model);
    const reasoningEffort =
      catalogModel && "reasoning_efforts" in catalogModel
        ? normalizeReasoningEffort(rest.reasoningEffort ?? "", catalogModel)
        : rest.reasoningEffort;
    return {
      settings: {
        client,
        settings: {
          ...rest,
          model,
          ...(reasoningEffort ? { reasoningEffort } : {}),
        },
      },
      connection: {
        client,
        connection: codexConnectionFor(endpoint, relayUrl),
      },
    };
  }

  /** 保存设置时去掉连接字段：地址与 Key 由 connection 承载，不属于设置。 */
  function settingsPayload(
    client: AgentClient,
    settings: AgentSettings["settings"],
  ) {
    if (client === "openCode") {
      const { baseUrl, endpointToken, ...rest } = settings as OpenCodeSettings;
      return rest;
    }
    const { endpointName, baseUrl, ...rest } = settings as CodexSettings;
    return rest;
  }

  function buildOpenCodeRequest(
    settings: AgentSettings["settings"],
    model: string,
    endpoint: RelayEndpoint,
    relayUrl: string,
    client: AgentClient,
  ) {
    const { baseUrl, endpointToken, ...rest } = settings as OpenCodeSettings;
    return {
      settings: { client, settings: { ...rest, model } },
      connection: {
        client,
        connection: openCodeConnectionFor(endpoint, relayUrl),
      },
    };
  }

  async function sync(matches?: (endpointId: string) => boolean) {
    const relayUrl = bootstrap.value?.relay_url;
    if (!relayUrl || catalogStatus.value !== "ready") {
      // 目录没就绪时不做半套对账，等下一次触发。
      return SYNC_HOSTS.map((target) => ({
        client: target.client,
        status: "skipped" as const,
      }));
    }
    const [links, endpoints, statuses] = await Promise.all([
      invokeLocalCommand<AgentEndpointLinks>(
        "agent_endpoint_links_get",
        undefined,
        {
          notify: false,
          trackPending: false,
        },
      ),
      invokeCommand<RelayEndpoint[]>("endpoints_list"),
      invokeLocalCommand<AgentClientStatus[]>("agents_status", undefined, {
        notify: false,
        trackPending: false,
      }),
    ]);
    const outcomes: AgentSyncOutcome[] = [];
    for (const target of SYNC_HOSTS) {
      const outcome = await syncHost(
        target,
        endpoints,
        links,
        statuses,
        relayUrl,
        matches,
      );
      outcomes.push(outcome);
    }
    outcomes.forEach(notify);
    return outcomes;
  }

  /** 接入点被删除后：把它写进宿主配置的地址与 Key 一并清掉。 */
  async function clear(endpoint: RelayEndpoint) {
    const relayUrl = bootstrap.value?.relay_url ?? "";
    const [links, statuses] = await Promise.all([
      invokeLocalCommand<AgentEndpointLinks>(
        "agent_endpoint_links_get",
        undefined,
        { notify: false, trackPending: false },
      ),
      invokeLocalCommand<AgentClientStatus[]>("agents_status", undefined, {
        notify: false,
        trackPending: false,
      }),
    ]);
    const outcomes: AgentSyncOutcome[] = [];
    for (const target of SYNC_HOSTS) {
      const client = target.client;
      if (!statuses.find((status) => status.client === client)?.installed) {
        outcomes.push({ client, status: "skipped" });
        continue;
      }
      const settings = await invokeLocalCommand<AgentSettings>(
        "agent_settings_get",
        { client },
        { notify: false, trackPending: false },
      );
      // 只清引用这个接入点的宿主：记录命中，或按名字/token 认领得上。
      const matched = linkedEndpoint({
        client,
        settings: settings.settings,
        links,
        endpoints: [endpoint],
        relayUrl,
      });
      if (!matched) {
        outcomes.push({ client, status: "not_linked" });
        continue;
      }
      const changed = await invokeLocalCommand<boolean>(
        "agent_settings_save",
        {
          settings: {
            client,
            settings: settingsPayload(client, settings.settings),
          },
          connection: null,
        },
        { notify: true, trackPending: false },
      );
      outcomes.push({ client, status: changed ? "synced" : "unchanged" });
      if (changed) {
        notifications.success("已移除已删除接入点的地址与 Key。", {
          title: `${clientLabel(client)} 已解除接入`,
        });
      }
    }
    return outcomes;
  }

  function track(work: () => Promise<AgentSyncOutcome[]>) {
    const running = pendingSync ?? Promise.resolve([]);
    const next = (async () => {
      await running;
      return await work();
    })();
    pendingSync = next;
    return next.finally(() => {
      if (pendingSync === next) pendingSync = null;
    });
  }

  return {
    /** 保存接入点或重置 Token 后：只对账引用这个接入点的宿主。 */
    syncEndpoint(endpointId: string) {
      return track(() => sync((id) => id === endpointId));
    },
    /** 启动、切换服务地址后：对账所有记录在案的宿主。 */
    syncAll() {
      return track(() => sync());
    },
    /** 删除接入点后：清掉宿主配置里留下的地址与 Key。 */
    clearEndpoint(endpoint: RelayEndpoint) {
      return track(() => clear(endpoint));
    },
  };
}
