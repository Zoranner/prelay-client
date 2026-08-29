import type { AgentClient, ExtensionCatalogKind } from "~/stores/relay";

const codexHostAgents: AgentClient[] = ["codexCli", "chatgpt"];
const allAgents: AgentClient[] = ["codexCli", "chatgpt", "openCode"];

export function linkedAgentsForExtension(
  kind: ExtensionCatalogKind,
): AgentClient[] {
  return kind === "skill" ? allAgents : codexHostAgents;
}

export function synchronizeExtensionInstallSelection({
  detected,
  kind,
  next,
  previous,
}: {
  detected: AgentClient[];
  kind: ExtensionCatalogKind;
  next: AgentClient[];
  previous: AgentClient[];
}): AgentClient[] {
  const linked = linkedAgentsForExtension(kind);
  const nextSelection = new Set(next);
  const previousSelection = new Set(previous);
  const detectedAgents = new Set(detected);
  const changedLinkedAgent = linked.find(
    (agent) => nextSelection.has(agent) !== previousSelection.has(agent),
  );
  const unlinkedSelection = next.filter((agent) => !linked.includes(agent));

  if (changedLinkedAgent && !nextSelection.has(changedLinkedAgent)) {
    return unlinkedSelection;
  }

  const linkedSelection = changedLinkedAgent
    ? linked.filter((agent) => detectedAgents.has(agent))
    : linked.filter((agent) => nextSelection.has(agent));

  return [...unlinkedSelection, ...linkedSelection];
}
