import type { AgentClient, CatalogLanguageModelResponse } from "~/types/relay";

type AgentConnectionModel = {
  modelName: string;
  upstreamModel: string;
  catalogModel?: CatalogLanguageModelResponse;
};

export type CodexSettingsDraft = {
  endpoint: string;
  model: string;
  reasoningEffort: string;
  personality: string;
  webSearch: boolean;
  sandbox: string;
  maxThreads: number;
  maxDepth: number;
  jobMaxRuntimeSeconds: number;
  networkAccess: boolean;
  shellEnvironmentInherit: string;
  windowsSandbox: string;
  features: {
    memories: boolean;
    goals: boolean;
    workspaceDependencies: boolean;
  };
  rules: string;
};

export type ChatGptSettingsDraft = CodexSettingsDraft;

export type AgentConfiguration = {
  codexCli: CodexSettingsDraft;
  chatgpt: ChatGptSettingsDraft;
};

export type AgentSettingsSaveRequest = {
  settings: {
    client: AgentClient;
    settings: Record<string, unknown>;
  };
  connection: {
    client: AgentClient;
    connection: Record<
      string,
      | string
      | undefined
      | AgentConnectionModel[]
      | CatalogLanguageModelResponse[]
    >;
  } | null;
};

function createCodexSettingsDraft(): CodexSettingsDraft {
  return {
    endpoint: "",
    model: "",
    reasoningEffort: "",
    personality: "pragmatic",
    webSearch: true,
    sandbox: "workspace-write",
    maxThreads: 16,
    maxDepth: 1,
    jobMaxRuntimeSeconds: 1800,
    networkAccess: true,
    shellEnvironmentInherit: "all",
    windowsSandbox: "unelevated",
    features: {
      memories: true,
      goals: true,
      workspaceDependencies: false,
    },
    rules: "",
  };
}

export function createAgentConfiguration(): AgentConfiguration {
  return {
    codexCli: createCodexSettingsDraft(),
    chatgpt: createCodexSettingsDraft(),
  };
}

export function copyAgentClientSettings(
  source: AgentConfiguration,
  target: AgentConfiguration,
  client: AgentClient,
) {
  if (client === "codexCli") {
    Object.assign(target.codexCli, source.codexCli, {
      features: { ...source.codexCli.features },
    });
  } else if (client === "chatgpt") {
    Object.assign(target.chatgpt, source.chatgpt, {
      features: { ...source.chatgpt.features },
    });
  }
}

export function codexSettingsPayload(
  settings: CodexSettingsDraft | ChatGptSettingsDraft,
) {
  const { reasoningEffort, ...payload } = settings;

  if (reasoningEffort.trim() !== "") {
    return {
      ...payload,
      reasoningEffort,
      features: { ...payload.features },
    };
  }

  return { ...payload, features: { ...payload.features } };
}
