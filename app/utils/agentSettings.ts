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

export interface ChatGptSettingsDraft extends CodexSettingsDraft {}

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
