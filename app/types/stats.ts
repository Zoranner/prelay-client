export interface ModelStats {
  model_requested: string | null;
  model_requested_display_name?: string | null;
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  input_tokens: number;
  output_tokens: number;
  average_latency_ms: number | null;
}

export interface ProviderStats {
  provider_type: string | null;
  provider_name: string | null;
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  input_tokens: number;
  output_tokens: number;
  average_latency_ms: number | null;
  average_first_token_ms: number | null;
}
