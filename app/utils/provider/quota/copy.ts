import type {
  ProviderQuotaMode,
  QuotaBalancePart,
  QuotaErrorCode,
  QuotaWindow,
} from "../../../types/quota";

function currencySymbol(currency: string) {
  if (currency === "CNY") return "¥";
  if (currency === "USD") return "$";
  return currency ? `${currency} ` : "";
}

export function formatBalance(parts: QuotaBalancePart[]) {
  return parts
    .map((part) => {
      const amount = part.amount.trim();
      const negative = amount.startsWith("-");
      return `${negative ? "-" : ""}${currencySymbol(part.currency)}${
        negative ? amount.slice(1) : amount
      }`;
    })
    .join(" · ");
}

export function errorCopy(code: QuotaErrorCode): string {
  switch (code) {
    case "unsupported":
      return "未接入";
    case "unconfigured":
      return "待配置";
    case "unauthorized":
      return "授权失败";
    case "unreachable":
      return "无法连接";
    case "invalid_response":
      return "返回异常";
    default:
      return "查询失败";
  }
}

function formatSpan(span: number | null | undefined) {
  if (!span) return "额度";
  if (span % 86_400 === 0) return `${span / 86_400} 天`;
  if (span % 3_600 === 0) return `${span / 3_600} 小时`;
  return `${Math.round(span / 60)} 分钟`;
}

export function windowCopy(window: QuotaWindow, mode: ProviderQuotaMode) {
  if (window.kind === "rolling") return formatSpan(window.span_secs);
  if (window.kind === "day") return mode === "shared_limited" ? "当天" : "1 天";
  if (window.kind === "week") {
    return mode === "shared_limited" ? "本周" : "周";
  }
  if (window.kind === "month") {
    return mode === "shared_limited" ? "本月" : "月";
  }
  return "额度";
}

export function windowSlot(window: QuotaWindow, mode: ProviderQuotaMode) {
  if (mode === "shared_limited") return window.kind === "day" ? 0 : 1;
  if (window.kind === "rolling" || window.kind === "day") return 0;
  if (window.kind === "week") return 1;
  return 2;
}
