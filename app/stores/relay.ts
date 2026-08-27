export interface BootstrapState {
  relay_url?: string;
  display_name: string;
  avatar_seed: string;
  has_device_credential: boolean;
}

export interface ProviderModel {
  id: string;
  provider_id: string;
  model_name: string;
  created_at: string;
}

export type UpstreamProtocol =
  | "responses"
  | "openai"
  | "anthropic"
  | "images_generations";

export interface ProviderProtocolBaseUrls {
  responses?: string | null;
  openai?: string | null;
  anthropic?: string | null;
  images_generations?: string | null;
}

export interface ProviderCapabilities {
  upstream_protocols?: string[] | null;
  protocol_base_urls?: ProviderProtocolBaseUrls | null;
  tool_calls?: boolean | null;
  reasoning?: boolean | null;
  tool_choice?: boolean | null;
  parallel_tool_calls?: boolean | null;
  system_messages?: boolean | null;
  structured_outputs?: boolean | null;
  streaming_usage?: boolean | null;
  max_context_tokens?: number | null;
  max_output_tokens?: number | null;
}

export interface Provider {
  id: string;
  name: string;
  provider_type: string;
  base_url: string;
  api_key: string;
  api_key_masked: string;
  capabilities: ProviderCapabilities;
  upstream_protocols: string[];
  models: ProviderModel[];
  created_at: string;
}

export interface EndpointModel {
  id?: string;
  endpoint_id?: string;
  model_name: string;
  provider_id: string;
  upstream_model: string;
  created_at?: string;
}

export interface RelayEndpoint {
  id: string;
  name: string;
  protocol: string;
  token: string;
  models: EndpointModel[];
  created_at: string;
}

export interface StatsOverview {
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  input_tokens: number;
  total_input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_write_tokens: number;
  average_latency_ms: number | null;
}

export interface TokenUsageTimelinePoint {
  bucket: string;
  input_tokens: number;
  total_input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_write_tokens: number;
}

export type StatsRange =
  | "today"
  | "yesterday"
  | "this_week"
  | "last_week"
  | "this_month"
  | "last_month"
  | "this_year"
  | "last_year"
  | "all";

export interface ModelStats {
  model_requested: string | null;
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  input_tokens: number;
  output_tokens: number;
  estimated_cost: number | null;
  average_latency_ms: number | null;
}

export interface ProviderStats extends ModelStats {
  provider_id: string | null;
  provider_name: string | null;
  average_first_token_ms: number | null;
}

export interface RequestLog {
  id: string;
  created_at: string;
  protocol_in: string | null;
  protocol_upstream: string | null;
  endpoint_name: string | null;
  provider_name: string | null;
  model_requested: string | null;
  model_upstream: string | null;
  status: string;
  http_status: number | null;
  error_code: string | null;
  error_message: string | null;
  input_tokens: number | null;
  output_tokens: number | null;
  is_streaming: boolean | null;
  first_token_ms: number | null;
  cache_read_tokens: number | null;
  cache_write_tokens: number | null;
  latency_ms: number | null;
  upstream_request_id: string | null;
  metadata_json: string | null;
}

export type AgentClient = "codexCli" | "chatgpt" | "claudeCode";
export type AgentItemKind = "mcp" | "skill" | "plugin";
export type AgentItemStatus = "enabled" | "disabled" | "error";

export interface AgentItem {
  kind: AgentItemKind;
  name: string;
  version: string | null;
  sourcePath: string;
  status: AgentItemStatus;
  errorMessage: string | null;
}

export interface AgentClientItems {
  client: AgentClient;
  version: string | null;
  items: AgentItem[];
}

export interface AgentItemsSnapshot {
  clients: AgentClientItems[];
}

export interface AgentClientVersion {
  client: AgentClient;
  version: string | null;
}

export type ExtensionKind = "rule" | "plugin" | "mcp" | "skill";

export interface ExtensionPackage {
  repository: string;
  commitSha: string;
  name: string;
  version: string;
  summary: string;
  kind: ExtensionKind;
  risk: string;
}

export interface ExtensionCatalogSnapshot {
  packages: ExtensionPackage[];
}

export interface ExtensionInstallAction {
  target: string;
  description: string;
}

export interface ExtensionInstallPreview {
  supported: boolean;
  message: string | null;
  actions: ExtensionInstallAction[];
}

export type CodexSettings = Partial<{
    endpointName: string;
    baseUrl: string;
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
    features: Partial<{
      memories: boolean;
      goals: boolean;
      workspaceDependencies: boolean;
    }>;
    rules: string;
}>;

export type ClaudeCodeSettings = Partial<{
    baseUrl: string;
    endpointToken: string;
    opusModel: string;
    sonnetModel: string;
    haikuModel: string;
    subagentModel: string;
    effort: string;
    language: string;
    permissionMode: string;
    rules: string;
}>;

export interface ChatGptSettings extends CodexSettings {}

export type AgentSettings =
  | { client: "codexCli"; settings: CodexSettings }
  | { client: "chatgpt"; settings: ChatGptSettings }
  | { client: "claudeCode"; settings: ClaudeCodeSettings };

export function useRelayStore() {
  const bootstrap = useState<BootstrapState | null>("relay-bootstrap", () => null);

  function setBootstrap(value: BootstrapState) {
    bootstrap.value = value;
  }

  function clearBootstrap() {
    bootstrap.value = null;
  }

  return { bootstrap, setBootstrap, clearBootstrap };
}
