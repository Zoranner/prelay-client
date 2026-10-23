import type { AgentItem } from "~/stores/relay";

/** 扩展库装的 MCP 只有一条宿主配置，技能包才会连带包内全部 Skill。 */
export function agentUninstallDescription(item: AgentItem): string {
  if (item.package === null) {
    return "配置及相关本地文件将一并删除，且无法恢复。";
  }
  if (item.kind === "plugin") {
    return "卸载插件会清理包内全部 Skill、MCP 和 Hook，且无法恢复。";
  }
  if (item.kind === "hook") {
    return `该条目由扩展包“${item.package}”安装，卸载会移除包内全部 Hook，且无法恢复。`;
  }
  return item.kind === "mcp"
    ? `该条目由扩展包“${item.package}”安装，卸载会移除这条 MCP 配置，且无法恢复。`
    : `该条目属于扩展包“${item.package}”，卸载会移除包内全部 Skill，且无法恢复。`;
}
