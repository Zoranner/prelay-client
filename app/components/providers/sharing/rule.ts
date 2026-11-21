import type { EditableWindow } from "./types";

export type AccumulateCycle = "none" | "week" | "month";

export type QuotaRule = {
  amount: string;
  accumulate: AccumulateCycle;
};

/** 额度统一以亿为单位录入。 */
export const quotaUnitScale = 100_000_000;

export const accumulateOptions = [
  { label: "不累积", value: "none" },
  { label: "周内累积", value: "week" },
  { label: "月内累积", value: "month" },
] satisfies Array<{ label: string; value: AccumulateCycle }>;

export function quotaRuleLimit(rule: QuotaRule): number {
  const amount = Number(rule.amount);
  if (!Number.isFinite(amount) || amount <= 0) return 0;
  return Math.round(amount * quotaUnitScale);
}

export function ruleFromWindows(windows: EditableWindow[]): QuotaRule {
  const day = windows.find((window) => window.kind === "day");
  const week = windows.find((window) => window.kind === "week");
  const month = windows.find((window) => window.kind === "month");
  const limit = Number(day?.limit ?? "");
  const amount = limit > 0 ? formatAmount(limit) : "";
  const accumulate: AccumulateCycle = month?.enabled
    ? "month"
    : week?.enabled
      ? "week"
      : "none";
  return { amount, accumulate };
}

export function windowsFromRule(
  rule: QuotaRule,
  current: EditableWindow[],
): EditableWindow[] {
  const limit = quotaRuleLimit(rule);
  const dayAccumulate = rule.accumulate !== "none";

  return current.map((window) => {
    const enabled =
      window.kind === "day"
        ? true
        : window.kind === "week"
          ? rule.accumulate === "week"
          : rule.accumulate === "month";
    return {
      ...window,
      enabled,
      limit:
        enabled && limit > 0
          ? String(
              window.kind === "day"
                ? limit
                : window.kind === "week"
                  ? limit * 7
                  : limit * 30,
            )
          : "",
      // 日窗口承接昨天的剩余，周/月窗口只做本周期总额。
      accumulate: window.kind === "day" ? dayAccumulate : false,
      reset_time: "00:00",
      reset_weekday: 1,
      reset_day: 1,
    };
  });
}

function formatAmount(limit: number): string {
  const billions = limit / quotaUnitScale;
  return Number.isInteger(billions)
    ? String(billions)
    : String(Number(billions.toFixed(4)));
}
