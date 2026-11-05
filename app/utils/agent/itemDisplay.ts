import type { AgentItem } from "~/types/relay";

export function agentItemKindLabel(kind: AgentItem["kind"]) {
  return { plugin: "插件", mcp: "MCP", skill: "Skill", hook: "Hook" }[kind];
}

export function agentItemKindIcon(kind: AgentItem["kind"]) {
  return {
    plugin: "ph:puzzle-piece",
    mcp: "ph:terminal-window",
    skill: "ph:book-open-text",
    hook: "ph:webhooks-logo",
  }[kind];
}

export function agentSourceLabel(source: AgentItem["source"]) {
  return { personal: "个人", team: "团队", system: "内置" }[source];
}

export function agentSourceSemantic(source: AgentItem["source"]) {
  return source === "team"
    ? "primary"
    : source === "system"
      ? "info"
      : undefined;
}

export function hasUnresolvedError(item: AgentItem) {
  return item.package === null && Boolean(item.errorMessage);
}

export function agentItemKey(item: AgentItem) {
  if (item.kind === "hook" && item.hookHash) {
    return `hook:${item.hookHash}`;
  }
  return `${item.kind}:${item.name}:${item.sourcePath}`;
}

export function hookMatcher(item: AgentItem) {
  return item.kind === "hook" ? item.matcher : null;
}

export function agentUninstallDisabled(item: AgentItem, pending: boolean) {
  return (
    pending ||
    item.source === "system" ||
    (item.package === null && Boolean(item.errorMessage))
  );
}

export function agentUninstallLabel(item: AgentItem) {
  if (item.source === "system") return "内置技能不可卸载";
  if (hasUnresolvedError(item)) return "状态错误不可卸载";
  return item.kind === "hook" ? "移除 Hook" : "卸载";
}

export function agentUninstallTitle(item: AgentItem) {
  return item.kind === "hook" ? "移除 Hook" : `卸载 ${item.name}`;
}

export function agentUninstallMessage(item: AgentItem) {
  if (item.kind !== "hook") return `卸载“${item.name}”？`;
  const matcher = hookMatcher(item);
  return matcher
    ? `移除 ${item.name} 的匹配条件“${matcher}”？`
    : `移除 ${item.name} 的 Hook？`;
}
