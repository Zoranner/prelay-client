import type { ProviderVisibility } from "./relay";

export type ShareQuotaWindowKind = "day" | "week" | "month";

export type ProviderQuotaMode = "own" | "shared_unlimited" | "shared_limited";

export interface ShareQuotaWindow {
  window: ShareQuotaWindowKind;
  limit: number;
  accumulate: boolean;
  reset_time: string;
  reset_weekday: number | null;
  reset_day: number | null;
}

export interface ProviderShareUserInput {
  identity_id: string;
}

export interface ProviderShareUser {
  identity_id: string;
  display_name: string;
}

export interface ProviderSharing {
  visibility: ProviderVisibility;
  users: ProviderShareUser[];
  quota: ShareQuotaWindow[] | null;
}

export interface ProviderSharingInput {
  visibility: ProviderVisibility;
  users: ProviderShareUserInput[];
  quota: ShareQuotaWindow[] | null;
}

export interface QuotaWindow {
  label: string;
  span_secs?: number | null;
  used_percent: number;
  resets_at?: string | null;
  display?: string | null;
  amount?: string | null;
  limit?: string | null;
  unit?: string | null;
  aside: boolean;
  pool?: string | null;
}

export interface QuotaBalancePart {
  currency: string;
  amount: string;
}

export interface QuotaBalance {
  text: string;
  parts: QuotaBalancePart[];
  available: boolean;
}

export type QuotaErrorCode =
  | "unsupported"
  | "unconfigured"
  | "unauthorized"
  | "unreachable"
  | "invalid_response";

export interface QuotaError {
  code: QuotaErrorCode;
  message: string;
}

export interface ProviderQuota {
  plan?: string | null;
  windows?: QuotaWindow[];
  balance?: QuotaBalance | null;
  error?: QuotaError | null;
  read_at?: string | null;
  stale: boolean;
}

export interface ProviderQuotaResponse {
  provider_id: string;
  owner_identity_id: string;
  owner_display_name: string;
  mode: ProviderQuotaMode;
  quota: ProviderQuota;
  updated_at: string | null;
}
