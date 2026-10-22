import type {
  AgentClient,
  ExtensionArtifactSummary,
} from "~/stores/relay";

const codexHostAgents: AgentClient[] = ["codexCli", "chatgpt"];
const allAgents: AgentClient[] = ["codexCli", "chatgpt", "openCode"];

export function extensionSupportsClient(
  artifacts: ExtensionArtifactSummary[],
  client: AgentClient,
) {
  if (!artifacts.length) return client === "codexCli" || client === "chatgpt";
  if (artifacts.some(({ kind }) => kind !== "hook")) return true;
  return client === "codexCli";
}

export function linkedAgentsForExtension(
  artifacts: ExtensionArtifactSummary[],
): AgentClient[] {
  if (!artifacts.length) return codexHostAgents;
  if (artifacts.some(({ kind }) => kind === "skill" || kind === "mcp")) {
    return allAgents;
  }
  if (artifacts.every(({ kind }) => kind === "hook")) return ["codexCli"];
  return codexHostAgents;
}

export function synchronizeExtensionInstallSelection({
  detected,
  artifacts,
  next,
  previous,
}: {
  detected: AgentClient[];
  artifacts: ExtensionArtifactSummary[];
  next: AgentClient[];
  previous: AgentClient[];
}): AgentClient[] {
  const linked = linkedAgentsForExtension(artifacts);
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
