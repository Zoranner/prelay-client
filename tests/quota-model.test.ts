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
  ).toBe("本周累计");
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
  expect(shared.tooltip.profile).toContain("共享限额");

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

  const status = buildQuotaView(
    response("own", { error: { code: "unreachable", message: "x" } }),
  );
  expect(status.variant).toBe("status");
  if (status.variant !== "status") throw new Error("expected status view");
  expect(status.primary.value).toBe("无法连接");
});
