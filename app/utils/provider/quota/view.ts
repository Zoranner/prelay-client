import type {
  ProviderQuotaResponse,
  QuotaBalance,
  QuotaErrorCode,
  QuotaWindow,
} from "../../../types/quota";
import { formatTokens } from "../../stats/tokenFormat";

import { errorCopy, formatBalance, windowCopy, windowSlot } from "./copy";
import { formatCountdown, formatPercent, formatUpdatedAt } from "./format";
import {
  hasWindowAmount,
  remainingPercent,
  sortWindows,
  tightestWindow,
} from "./model";

export type QuotaTone = "primary" | "warning" | "danger" | "muted";

export interface RingSlot {
  key: string;
  label: string;
  percent: number;
  configured: boolean;
}

export interface QuotaText {
  label: string;
  value: string;
  tone: QuotaTone;
}

export interface QuotaRow {
  key: string;
  label: string;
  value: string;
  percent: number;
  meta: string;
  tone: QuotaTone;
}

export interface QuotaTooltip {
  profile: string;
  status: string;
  statusTone: "success" | "muted" | "danger";
  rows: QuotaRow[];
  message: string | null;
  footer: string[];
}

export interface QuotaPlanView {
  variant: "plan";
  slots: RingSlot[];
  primary: QuotaText;
  secondary?: QuotaText;
  stale: boolean;
  tooltip: QuotaTooltip;
}

export interface QuotaSharedView {
  variant: "shared";
  slots: RingSlot[];
  primary: QuotaText;
  stale: boolean;
  tooltip: QuotaTooltip;
}

export interface QuotaBalanceView {
  variant: "balance";
  balance: QuotaBalance;
  primary: QuotaText;
  stale: boolean;
  tooltip: QuotaTooltip;
}

export interface QuotaStatusView {
  variant: "status";
  code: QuotaErrorCode | null;
  primary: QuotaText;
  stale: boolean;
  tooltip: QuotaTooltip;
}

export type QuotaView =
  QuotaPlanView | QuotaSharedView | QuotaBalanceView | QuotaStatusView;

export function toneFor(percent: number): QuotaTone {
  if (percent <= 15) return "danger";
  if (percent <= 35) return "warning";
  return "primary";
}

function windowKey(window: QuotaWindow) {
  return `${window.kind}-${window.span_secs ?? "none"}-${window.resets_at ?? ""}`;
}

function slotsFor(
  windows: QuotaWindow[],
  mode: ProviderQuotaResponse["mode"],
  count: number,
) {
  const slots: RingSlot[] = Array.from({ length: count }, (_, index) => ({
    key: `empty-${index}`,
    label: "",
    percent: 0,
    configured: false,
  }));
  for (const window of windows) {
    const index = windowSlot(window, mode);
    if (index >= count || slots[index]?.configured) continue;
    slots[index] = {
      key: windowKey(window),
      label: windowCopy(window, mode),
      percent: remainingPercent(window),
      configured: true,
    };
  }
  return slots;
}

function planLabel(plan: string) {
  const id = plan.toLowerCase();
  if (id.includes("goat")) return "GOAT";
  if (id.includes("ultra")) return "Ultra";
  if (id.includes("max-20x")) return "Max 20×";
  if (id.includes("max-10x")) return "Max 10×";
  if (id.includes("individual-max")) return "Max";
  if (id.includes("individual-pro")) return "Pro";
  if (id.includes("individual-go")) return "Go";
  if (id.includes("teams-pro") || id.includes("team-pro")) return "Team Pro";
  return plan;
}

function profileLabel(response: ProviderQuotaResponse) {
  const mode =
    response.mode === "shared_limited"
      ? "共享限额"
      : response.mode === "shared_unlimited"
        ? "共享"
        : "个人";
  const parts = [mode];
  if (response.quota.plan) parts.push(planLabel(response.quota.plan));
  else if (response.owner_display_name) parts.push(response.owner_display_name);
  return parts.join(" · ");
}

function statusFor(response: ProviderQuotaResponse) {
  const error = response.quota.error;
  if (error) {
    const muted = error.code === "unsupported" || error.code === "unconfigured";
    return {
      label: errorCopy(error.code),
      tone: muted ? ("muted" as const) : ("danger" as const),
    };
  }
  if (response.quota.stale) {
    return { label: "旧读数", tone: "muted" as const };
  }
  return { label: "正常", tone: "success" as const };
}

function planRow(window: QuotaWindow, mode: ProviderQuotaResponse["mode"]) {
  const percent = remainingPercent(window);
  const meta = [window.display, formatCountdown(window.resets_at)]
    .filter(Boolean)
    .join(" · ");
  return {
    key: windowKey(window),
    label: windowCopy(window, mode),
    value: formatPercent(percent),
    percent,
    meta,
    tone: toneFor(percent),
  } satisfies QuotaRow;
}

function sharedValue(window: QuotaWindow) {
  return window.remaining != null
    ? formatTokens(window.remaining)
    : formatPercent(remainingPercent(window));
}

function sharedRow(window: QuotaWindow) {
  const percent = remainingPercent(window);
  const meta =
    hasWindowAmount(window) && window.remaining != null && window.limit != null
      ? `已用 ${formatTokens(Math.max(window.limit - window.remaining, 0))} / ${formatTokens(window.limit)}`
      : (window.display ?? "");
  return {
    key: windowKey(window),
    label: windowCopy(window, "shared_limited"),
    value: sharedValue(window),
    percent,
    meta: [meta, formatCountdown(window.resets_at)].filter(Boolean).join(" · "),
    tone: toneFor(percent),
  } satisfies QuotaRow;
}

function tooltip(
  response: ProviderQuotaResponse,
  rows: QuotaRow[],
  footer: string[],
  message: string | null = null,
): QuotaTooltip {
  const status = statusFor(response);
  return {
    profile: profileLabel(response),
    status: status.label,
    statusTone: status.tone,
    rows,
    message,
    footer,
  };
}

export function buildQuotaView(response: ProviderQuotaResponse): QuotaView {
  const quota = response.quota;
  const windows = quota.windows ?? [];
  const updated = formatUpdatedAt(response.updated_at);
  const stale = quota.stale === true;

  if (response.mode === "shared_limited") {
    const sorted = sortWindows(windows, response.mode);
    const summary =
      sorted.find((window) => window.kind === "day") ??
      tightestWindow(sorted, response.mode);
    const rows = sorted.map(sharedRow);
    return {
      variant: "shared",
      slots: slotsFor(sorted, response.mode, 2),
      primary: summary
        ? {
            label: windowCopy(summary, response.mode),
            value: sharedValue(summary),
            tone: toneFor(remainingPercent(summary)),
          }
        : { label: "共享限额", value: "", tone: "muted" },
      stale,
      tooltip: tooltip(response, rows, [updated]),
    };
  }

  if (windows.length) {
    const sorted = sortWindows(windows, response.mode);
    const tightest = tightestWindow(sorted, response.mode);
    const rows = sorted.map((window) => planRow(window, response.mode));
    const secondary =
      quota.balance && quota.balance.parts.length
        ? {
            label: "余额",
            value: formatBalance(quota.balance.parts),
            tone: quota.balance.available
              ? ("muted" as const)
              : ("danger" as const),
          }
        : undefined;
    return {
      variant: "plan",
      slots: slotsFor(sorted, response.mode, 3),
      primary: tightest
        ? {
            label: windowCopy(tightest, response.mode),
            value: formatPercent(remainingPercent(tightest)),
            tone: toneFor(remainingPercent(tightest)),
          }
        : { label: "额度", value: "—", tone: "muted" },
      secondary,
      stale,
      tooltip: tooltip(
        response,
        rows,
        [secondary ? `余额 ${secondary.value}` : "无余额", updated],
        quota.error ? errorCopy(quota.error.code) : null,
      ),
    };
  }

  if (quota.balance && quota.balance.parts.length) {
    const value = formatBalance(quota.balance.parts);
    return {
      variant: "balance",
      balance: quota.balance,
      primary: {
        label: "余额",
        value,
        tone: quota.balance.available ? "primary" : "danger",
      },
      stale,
      tooltip: tooltip(
        response,
        [],
        [updated],
        quota.balance.available ? null : "余额不可用",
      ),
    };
  }

  const code = quota.error?.code ?? null;
  return {
    variant: "status",
    code,
    primary: {
      label: "状态",
      value: code ? errorCopy(code) : "—",
      tone:
        code === "unsupported" || code === "unconfigured"
          ? "muted"
          : code
            ? "danger"
            : "muted",
    },
    stale,
    tooltip: tooltip(response, [], [updated], code ? null : "暂无额度读数"),
  };
}
