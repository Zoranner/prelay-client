import type { AgentClient, CatalogLanguageModelResponse } from "~/stores/relay";

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
  disableResponseStorage: boolean;
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

export type OpenCodeSettingsDraft = {
  endpoint: string;
  model: string;
  rules: string;
};

export type ClaudeCodeSettingsDraft = {
  endpoint: string;
  model: string;
  opusModel: string;
  sonnetModel: string;
  haikuModel: string;
  subagentModel: string;
  apiTimeoutMs: number | null;
  maxOutputTokens: number | null;
  toolSearchEnabled: boolean;
  nonessentialTrafficDisabled: boolean;
  rules: string;
};

export type AgentConfiguration = {
  codexCli: CodexSettingsDraft;
  chatgpt: ChatGptSettingsDraft;
  openCode: OpenCodeSettingsDraft;
  claudeCode: ClaudeCodeSettingsDraft;
};

export type AgentSettingsSaveRequest = {
  settings: {
    client: AgentClient;
    settings: Record<string, unknown>;
  };
  connection: {
    client: AgentClient;
    connection: Record<string, string | undefined | AgentConnectionModel[]>;
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
    disableResponseStorage: true,
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
    openCode: {
      endpoint: "",
      model: "",
      rules: "",
    },
    claudeCode: {
      endpoint: "",
      model: "",
      opusModel: "",
      sonnetModel: "",
      haikuModel: "",
      subagentModel: "",
      apiTimeoutMs: null,
      maxOutputTokens: null,
      toolSearchEnabled: false,
      nonessentialTrafficDisabled: false,
      rules: "",
    },
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
  } else if (client === "openCode") {
    Object.assign(target.openCode, source.openCode);
  } else {
    Object.assign(target.claudeCode, source.claudeCode);
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

export function openCodeSettingsPayload(settings: OpenCodeSettingsDraft) {
  return { ...settings };
}

export function claudeCodeSettingsPayload(settings: ClaudeCodeSettingsDraft) {
  return { ...settings };
}
