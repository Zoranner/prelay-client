import type { ProviderQuotaMode, QuotaWindow } from "../../../types/quota";

import { windowSlot } from "./copy";
import { clampPercent } from "./format";

export function remainingPercent(window: QuotaWindow) {
  return clampPercent(100 - window.used_percent);
}

export function sortWindows(
  windows: QuotaWindow[],
  mode: ProviderQuotaMode = "own",
) {
  return [...windows].sort((left, right) => {
    const slot = windowSlot(left, mode) - windowSlot(right, mode);
    if (slot !== 0) return slot;
    return (
      (left.span_secs ?? Number.MAX_SAFE_INTEGER) -
      (right.span_secs ?? Number.MAX_SAFE_INTEGER)
    );
  });
}

export function tightestWindow(
  windows: QuotaWindow[],
  mode: ProviderQuotaMode = "own",
) {
  if (!windows.length) return undefined;
  return sortWindows(windows, mode).reduce((tightest, current) =>
    remainingPercent(current) < remainingPercent(tightest) ? current : tightest,
  );
}

export function hasWindowAmount(window: QuotaWindow) {
  return window.remaining != null && window.limit != null;
}
