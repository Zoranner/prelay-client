import { errorText } from "~/utils/errors";
import { protocolLabel } from "~/utils/provider/templates";

export interface ProviderOperationResult {
  ok: boolean;
  protocol: string | null;
  latency_ms: number | null;
  first_token_ms: number | null;
  error_code?: string | null;
  error: string | null;
}

export interface ProviderOperationFeedback {
  success: boolean;
  message: string;
  metrics: string | null;
}

export function getProviderOperationFeedback(
  result: ProviderOperationResult,
): ProviderOperationFeedback {
  const metrics = [
    typeof result.latency_ms === "number"
      ? `延迟 ${result.latency_ms} ms`
      : null,
    typeof result.first_token_ms === "number"
      ? `首 Token ${result.first_token_ms} ms`
      : null,
  ].filter(Boolean);

  if (!result.ok) {
    return {
      success: false,
      message: result.error_code
        ? errorText({
            code: result.error_code,
            message: result.error?.trim() ?? "",
          })
        : "协议测试失败。",
      metrics: metrics.join("；") || null,
    };
  }

  const protocol = protocolLabel(result.protocol);

  return {
    success: true,
    message: result.protocol
      ? `${protocol === "-" ? result.protocol : protocol} 协议测试完成。`
      : "操作完成。",
    metrics: metrics.join("；") || null,
  };
}
