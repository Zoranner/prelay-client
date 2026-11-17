export type QuotaWindowKind = "five_hour" | "day" | "week" | "month";

export type ProviderQuotaMode = "own" | "shared_unlimited" | "shared_limited";

export interface ShareQuotaWindow {
  window: QuotaWindowKind;
  limit: number;
  accumulate: boolean;
  reset_time: string;
  reset_weekday: number | null;
  reset_day: number | null;
}

export interface ProviderShareInput {
  identity_id: string;
  quota: ShareQuotaWindow[] | null;
}

export interface ProviderShare {
  identity_id: string;
  display_name: string;
  quota: ShareQuotaWindow[] | null;
}

export interface ProviderSharing {
  shares: ProviderShare[];
  can_manage: boolean;
}

export interface ProviderSharingInput {
  shares: ProviderShareInput[];
}

export interface QuotaWindow {
  window: QuotaWindowKind;
  accumulate: boolean;
  remaining_percent: number;
  remaining: number | null;
  limit: number | null;
  used: number | null;
  carry_over: number | null;
  resets_at: string | null;
}

export interface QuotaBalance {
  available: boolean;
  amount: string;
  currency: string;
}

export type ProviderQuota =
  | { kind: "plan"; windows: QuotaWindow[] }
  | ({ kind: "balance" } & QuotaBalance)
  | { kind: "shared"; windows: QuotaWindow[] }
  | { kind: "unsupported" }
  | { kind: "error"; message: string };

export interface ProviderQuotaResponse {
  provider_id: string;
  owner_identity_id: string;
  owner_display_name: string;
  mode: ProviderQuotaMode;
  quota: ProviderQuota;
  updated_at: string | null;
}
