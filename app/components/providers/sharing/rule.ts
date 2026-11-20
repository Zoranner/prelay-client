import type { EditableWindow } from "./types";

export type AccumulateCycle = "none" | "day" | "week" | "month";
export type ResetCycle = "day" | "week" | "month" | "never";

export type QuotaRule = {
  amount: string;
  accumulate: AccumulateCycle;
  reset: ResetCycle;
};

/** 额度统一以亿为单位录入。 */
export const quotaUnitScale = 100_000_000;

export const accumulateOptions = [
  { label: "不累积", value: "none" },
  { label: "按日累积", value: "day" },
  { label: "按周累积", value: "week" },
  { label: "按月累积", value: "month" },
] satisfies Array<{ label: string; value: AccumulateCycle }>;

const resetRank: Record<ResetCycle, number> = {
  day: 0,
  week: 1,
  month: 2,
  never: 3,
};

const accumulateRank: Record<AccumulateCycle, number> = {
  none: -1,
  day: 0,
  week: 1,
  month: 2,
};

export function resetOptionsFor(accumulate: AccumulateCycle) {
  const minimum = accumulateRank[accumulate];
  return (
    [
      { label: "每天清零", value: "day" },
      { label: "每周清零", value: "week" },
      { label: "每月清零", value: "month" },
      { label: "不清零", value: "never" },
    ] satisfies Array<{ label: string; value: ResetCycle }>
  ).map((option) => ({
    ...option,
    // 不累积时额度当天作废，清零周期只能是每天。
    disabled:
      accumulate === "none"
        ? option.value !== "day"
        : option.value === "never" || resetRank[option.value] < minimum,
  }));
}

export function minimumResetFor(accumulate: AccumulateCycle): ResetCycle {
  if (accumulate === "week") return "week";
  if (accumulate === "month") return "month";
  return "day";
}

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
  const accumulate: AccumulateCycle = week?.enabled
    ? "week"
    : month?.enabled
      ? "month"
      : "none";
  const reset: ResetCycle = month?.enabled
    ? "month"
    : week?.enabled
      ? "week"
      : day?.enabled
        ? "day"
        : "never";
  return { amount, accumulate, reset };
}

export function windowsFromRule(
  rule: QuotaRule,
  current: EditableWindow[],
): EditableWindow[] {
  const limit = quotaRuleLimit(rule);
  const reset = rule.reset === "never" ? "month" : rule.reset;
  const weekEnabled =
    rule.accumulate === "week" ||
    rule.accumulate === "month" ||
    reset === "week" ||
    reset === "month";
  const monthEnabled = rule.accumulate === "month" || reset === "month";

  return current.map((window) => {
    const enabled =
      window.kind === "day"
        ? true
        : window.kind === "week"
          ? weekEnabled
          : monthEnabled;
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
      accumulate: false,
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
