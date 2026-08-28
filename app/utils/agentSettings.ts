export type CodexSettingsDraft = {
  endpoint: string;
  customBaseUrl: string;
  customToken: string;
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

export type AgentConfiguration = {
  codexCli: CodexSettingsDraft;
  chatgpt: ChatGptSettingsDraft;
  openCode: OpenCodeSettingsDraft;
};

export type AgentSettingsSaveRequest = {
  settings: {
    client: AgentClient;
    settings: Record<string, unknown>;
  };
  connection: {
    client: AgentClient;
    connection: Record<string, string | undefined>;
  } | null;
};

function createCodexSettingsDraft(): CodexSettingsDraft {
  return {
    endpoint: "",
    customBaseUrl: "",
    customToken: "",
    model: "",
    reasoningEffort: "high",
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
  } else {
    Object.assign(target.openCode, source.openCode);
  }
}

export function codexSettingsPayload(
  settings: CodexSettingsDraft | ChatGptSettingsDraft,
) {
  const { customToken, ...payload } = settings;
  return { ...payload, features: { ...payload.features } };
}

export function openCodeSettingsPayload(settings: OpenCodeSettingsDraft) {
  return { ...settings };
}
import type { AgentClient } from "~/stores/relay";
