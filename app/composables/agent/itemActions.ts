import { useConfirm, useNotification } from "@stellar/ui";

import type { AgentClient, AgentItem } from "~/types/relay";
import { agentUninstallDescription } from "~/utils/agent/uninstall";

type AgentItemRemoval = {
  client: AgentClient;
  kind: AgentItem["kind"];
  name: string;
  sourcePath: string;
};

export function useAgentItemActions(options: {
  client: Readonly<Ref<AgentClient>>;
  refreshClient: (client: AgentClient) => Promise<void>;
  remove: (request: AgentItemRemoval) => Promise<unknown>;
}) {
  const { confirm } = useConfirm();
  const notifications = useNotification();
  const kindLabels = {
    plugin: "插件",
    mcp: "MCP",
    skill: "Skill",
    hook: "Hook",
  } as const;

  async function uninstall(item: AgentItem) {
    const managed = item.package !== null;
    const confirmed = await confirm({
      title: `卸载 ${item.name}`,
      message: `卸载“${item.name}”？`,
      description: agentUninstallDescription(item),
      confirmText: "卸载",
      danger: true,
    });
    if (!confirmed) return;
    await options.remove({
      client: options.client.value,
      kind: item.kind,
      name: item.name,
      sourcePath: item.sourcePath,
    });
    await options.refreshClient(options.client.value);
    notifications.success(
      managed ? "扩展包已卸载" : `${kindLabels[item.kind]}已卸载`,
    );
  }

  return { uninstall };
}
