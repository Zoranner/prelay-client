import { expect, test } from "bun:test";

import type {
  ProviderQuota,
  ProviderQuotaMode,
  ProviderQuotaResponse,
  QuotaWindow,
} from "../app/types/quota";
import {
  errorCopy,
  windowCopy,
  windowSlot,
} from "../app/utils/provider/quota/copy";
import {
  formatCountdown,
  formatUpdatedAt,
} from "../app/utils/provider/quota/format";
import { tightestWindow } from "../app/utils/provider/quota/model";
import { buildQuotaView } from "../app/utils/provider/quota/view";

const window = (patch: Partial<QuotaWindow>): QuotaWindow => ({
  kind: "rolling",
  span_secs: 18_000,
  used_percent: 10,
  resets_at: null,
  remaining: null,
  limit: null,
  unit: null,
  display: null,
  ...patch,
});

test("窗口文案由类型和跨度决定", () => {
  expect(windowCopy(window({}), "own")).toBe("5 小时");
  expect(
    windowCopy(window({ kind: "day", span_secs: 86_400 }), "shared_limited"),
  ).toBe("当天");
  expect(
    windowCopy(window({ kind: "week", span_secs: 604_800 }), "shared_limited"),
  ).toBe("本周");
  expect(windowCopy(window({ kind: "week", span_secs: 604_800 }), "own")).toBe(
    "周",
  );
  expect(windowCopy(window({ kind: "month", span_secs: null }), "own")).toBe(
    "月",
  );
});

test("圆环按窗口类型分槽", () => {
  expect(windowSlot(window({}), "own")).toBe(0);
  expect(windowSlot(window({ kind: "day" }), "shared_limited")).toBe(0);
  expect(windowSlot(window({ kind: "week" }), "own")).toBe(1);
  expect(windowSlot(window({ kind: "month" }), "own")).toBe(2);
});

test("最紧窗口按剩余比例取最小", () => {
  const day = window({ kind: "day", used_percent: 5 });
  const week = window({ kind: "week", used_percent: 40 });
  expect(tightestWindow([day, week])?.kind).toBe("week");
});

test("错误文案由 code 决定", () => {
  expect(errorCopy("unreachable")).toBe("无法连接");
  expect(errorCopy("unconfigured")).toBe("待配置");
});

test("倒计时统一使用紧凑单位", () => {
  const at = (ms: number) => new Date(Date.now() + ms).toISOString();
  expect(formatCountdown(at(54 * 60_000))).toMatch(/^\d+m 后重置$/);
  expect(formatCountdown(at(5 * 3_600_000 + 17 * 60_000))).toMatch(
    /^\d+h \d+m 后重置$/,
  );
  expect(formatCountdown(at(10 * 86_400_000 + 19 * 3_600_000))).toMatch(
    /^\d+d \d+h 后重置$/,
  );
});

test("更新时间使用最后一次成功读数并标明最后更新", () => {
  expect(formatUpdatedAt(null)).toBe("最后更新时间未知");
  expect(formatUpdatedAt("2026-01-01T06:24:00Z")).toMatch(/^最后更新 /);
});

const response = (
  mode: ProviderQuotaMode,
  quota: Partial<ProviderQuota>,
): ProviderQuotaResponse => ({
  provider_id: "p",
  owner_identity_id: "o",
  owner_display_name: "owner",
  mode,
  quota: { windows: [], balance: null, error: null, stale: false, ...quota },
  updated_at: null,
});

test("视图模型统一 plan、shared、balance 和 status", () => {
  const plan = buildQuotaView(
    response("own", {
      windows: [
        window({ kind: "rolling" }),
        window({ kind: "week" }),
        window({ kind: "month" }),
      ],
    }),
  );
  expect(plan.variant).toBe("plan");
  if (plan.variant !== "plan") throw new Error("expected plan view");
  expect(plan.slots).toHaveLength(3);
  expect(plan.tooltip.balance).toBeNull();
  expect(plan.tooltip.rows[0]?.meta).toBe("重置时间未知");
  expect(plan.tooltip.profile).toBe("个人 · owner");

  const planWithName = buildQuotaView(
    response("own", {
      plan: "individual-goat",
      windows: [window({})],
    }),
  );
  expect(planWithName.tooltip.profile).toBe("个人 · owner");

  const idle = buildQuotaView(
    response("own", {
      windows: [window({ used_percent: 0, resets_at: null })],
    }),
  );
  expect(idle.variant).toBe("plan");
  if (idle.variant !== "plan") throw new Error("expected plan view");
  expect(idle.tooltip.rows[0]?.meta).toBe("本窗口暂无用量");

  const planWithBalance = buildQuotaView(
    response("own", {
      windows: [
        window({ kind: "rolling" }),
        window({ kind: "week" }),
        window({ kind: "month" }),
      ],
      balance: {
        parts: [{ currency: "USD", amount: "0.06" }],
        available: true,
      },
    }),
  );
  expect(planWithBalance.variant).toBe("plan");
  expect(planWithBalance.tooltip.balance?.value).toBe("$0.06");
  expect(planWithBalance.tooltip.footer).toEqual(["最后更新时间未知"]);

  const shared = buildQuotaView(
    response("shared_limited", {
      windows: [
        window({ kind: "day", remaining: 100, limit: 200, unit: "tokens" }),
        window({ kind: "week", remaining: 500, limit: 1000, unit: "tokens" }),
      ],
    }),
  );
  expect(shared.variant).toBe("shared");
  if (shared.variant !== "shared") throw new Error("expected shared view");
  expect(shared.slots).toHaveLength(2);
  expect(shared.slots[1].configured).toBe(true);
  expect(shared.primary.label).toBe("当天");
  expect(shared.primary.value).toBe("100");
  expect(shared.tooltip.profile).toBe("共享 · owner");

  const balance = buildQuotaView(
    response("own", {
      balance: {
        parts: [{ currency: "USD", amount: "5.00" }],
        available: true,
      },
    }),
  );
  expect(balance.variant).toBe("balance");
  if (balance.variant !== "balance") throw new Error("expected balance view");
  expect(balance.primary.value).toBe("$5.00");
  expect(balance.tooltip.balance?.label).toBe("余额");
  expect(balance.tooltip.balance?.value).toBe("$5.00");
  expect(balance.tooltip.message).toBeNull();

  const negative = buildQuotaView(
    response("own", {
      balance: {
        parts: [{ currency: "CNY", amount: "-0.11" }],
        available: false,
      },
    }),
  );
  expect(negative.variant).toBe("balance");
  if (negative.variant !== "balance") throw new Error("expected balance view");
  expect(negative.primary.value).toBe("-¥0.11");
  expect(negative.tooltip.balance?.tone).toBe("danger");
  expect(negative.tooltip.message).toBeNull();

  const status = buildQuotaView(
    response("own", { error: { code: "unreachable", message: "x" } }),
  );
  expect(status.variant).toBe("status");
  if (status.variant !== "status") throw new Error("expected status view");
  expect(status.primary.value).toBe("无法连接");
});
