import type { ShareQuotaAccumulate, ShareQuotaRule } from "~/types/quota";

export type QuotaRule = {
  amount: string;
  accumulate: ShareQuotaAccumulate;
};

/** 额度统一以亿为单位录入。 */
export const quotaUnitScale = 100_000_000;

export const accumulateOptions = [
  { label: "不累积", value: "none" },
  { label: "周内累积", value: "week" },
  { label: "月内累积", value: "month" },
] satisfies Array<{ label: string; value: ShareQuotaAccumulate }>;

export function quotaRuleAmount(rule: QuotaRule): number {
  const amount = Number(rule.amount);
  if (!Number.isFinite(amount) || amount <= 0) return 0;
  return Math.round(amount * quotaUnitScale);
}

export function ruleFromQuota(quota: ShareQuotaRule | null): QuotaRule {
  if (!quota || quota.amount <= 0) {
    return { amount: "", accumulate: "none" };
  }
  return {
    amount: formatAmount(quota.amount),
    accumulate: quota.accumulate,
  };
}

export function quotaFromRule(rule: QuotaRule): ShareQuotaRule | null {
  const amount = quotaRuleAmount(rule);
  if (amount <= 0) return null;
  return { amount, accumulate: rule.accumulate };
}

function formatAmount(amount: number): string {
  const billions = amount / quotaUnitScale;
  return Number.isInteger(billions)
    ? String(billions)
    : String(Number(billions.toFixed(8)));
}
