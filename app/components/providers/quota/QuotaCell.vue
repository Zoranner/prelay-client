<script setup lang="ts">
defineOptions({ name: "ProviderQuotaCell" });

import { ProgressBar, Tooltip } from "@stellar/ui";
import type { ProviderQuotaResponse, QuotaWindow } from "~/types/quota";
import { formatTokens } from "~/utils/stats/tokenFormat";
import QuotaPlanRings from "./PlanRings.vue";

const props = defineProps<{
  quota: ProviderQuotaResponse | null;
  loading?: boolean;
}>();

const entry = computed(() => props.quota?.quota ?? null);
const windows = computed<QuotaWindow[]>(() =>
  (entry.value?.windows ?? []).filter((window) => !window.aside),
);
const balance = computed(() => entry.value?.balance ?? null);
const error = computed(() => entry.value?.error ?? null);
const sharedLimited = computed(() => props.quota?.mode === "shared_limited");
const stale = computed(() => entry.value?.stale === true);

function clampPercent(value: number) {
  return Math.max(0, Math.min(100, Number.isFinite(value) ? value : 0));
}

function remainingPercent(window: QuotaWindow) {
  return clampPercent(100 - window.used_percent);
}

const tightest = computed(() => {
  if (!windows.value.length) return 100;
  return Math.min(...windows.value.map(remainingPercent));
});

const progressSemantic = computed(() => {
  if (tightest.value <= 15) return "error" as const;
  if (tightest.value <= 35) return "warning" as const;
  return "success" as const;
});

const hasContent = computed(
  () => windows.value.length > 0 || balance.value !== null,
);

const statusText = computed(() => {
  if (props.loading && !props.quota) return "查询中";
  if (!props.quota) return "—";
  if (error.value) {
    switch (error.value.code) {
      case "unsupported":
        return "未接入";
      case "unconfigured":
        return "待配置";
      default:
        return "查询失败";
    }
  }
  return hasContent.value ? "" : "—";
});

function formatPercent(value: number) {
  const percent = clampPercent(value);
  return `${percent >= 99.5 ? 100 : Math.round(percent)}%`;
}

function formatReset(value: string | null | undefined) {
  if (!value) return "未知";
  return new Date(value).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatAmount(window: QuotaWindow) {
  if (window.amount == null) return window.display ?? "";
  const numeric = Number(window.amount);
  if (Number.isFinite(numeric) && (!window.unit || window.unit === "tokens")) {
    return formatTokens(numeric);
  }
  return window.unit ? `${window.amount} ${window.unit}` : window.amount;
}
</script>

<template>
  <Tooltip v-if="quota && (hasContent || error)">
    <div class="quota-cell">
      <template v-if="windows.length">
        <ProgressBar
          v-if="sharedLimited"
          class="quota-cell__bar"
          :value="tightest"
          :max="100"
          :show-header="false"
          size="small"
          :semantic="progressSemantic"
        />
        <QuotaPlanRings v-else :windows="windows" />
        <span
          class="quota-cell__value"
          :class="{ 'quota-cell__value--low': tightest <= 15 }"
        >
          {{ formatPercent(tightest) }}
        </span>
      </template>
      <span
        v-if="balance"
        class="quota-cell__balance"
        :class="{ 'quota-cell__balance--low': !balance.available }"
      >
        {{ balance.text }}
      </span>
      <span v-if="stale" class="quota-cell__stale">旧</span>
    </div>

    <template #content>
      <div class="quota-tooltip">
        <div
          v-for="window in windows"
          :key="window.label"
          class="quota-tooltip__row"
        >
          <span>{{ window.label }}</span>
          <strong>{{ formatPercent(remainingPercent(window)) }}</strong>
          <small>
            <template v-if="window.amount != null">
              剩 {{ formatAmount(window) }}
              <template v-if="window.display"> · {{ window.display }}</template>
              ·
            </template>
            <template v-else-if="window.display">
              {{ window.display }} ·
            </template>
            重置 {{ formatReset(window.resets_at) }}
            <template v-if="window.pool"> · {{ window.pool }}</template>
          </small>
        </div>
        <div v-if="balance" class="quota-tooltip__row">
          <span>可用余额</span>
          <strong>{{ balance.text }}</strong>
          <small>{{
            balance.available ? "账户可用" : "余额不足或不可用"
          }}</small>
        </div>
        <small v-if="error">{{ error.message }}</small>
        <small v-if="stale">显示的是上次成功读数</small>
        <small v-if="quota.updated_at">
          更新 {{ formatReset(quota.updated_at) }}
        </small>
      </div>
    </template>
  </Tooltip>
  <span
    v-else
    class="quota-cell__status"
    :class="{ 'quota-cell__status--muted': !loading }"
    :title="error?.message"
  >
    {{ statusText }}
  </span>
</template>

<style scoped>
.quota-cell {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-xs);
}

.quota-cell__bar {
  width: 42px;
  flex: 0 0 auto;
}

.quota-cell__value,
.quota-cell__balance {
  overflow: hidden;
  color: var(--st-text-primary);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.quota-cell__balance {
  max-width: 72px;
  color: var(--st-text-secondary);
  font-size: 11px;
}

.quota-cell__value--low,
.quota-cell__balance--low {
  color: var(--st-danger);
}

.quota-cell__stale {
  color: var(--st-text-muted);
  font-size: 10px;
}

.quota-cell__status {
  color: var(--st-text-secondary);
  font-size: 12px;
}

.quota-cell__status--muted {
  color: var(--st-text-muted);
}

.quota-tooltip {
  display: grid;
  min-width: 200px;
  gap: 6px;
}

.quota-tooltip__row {
  display: grid;
  grid-template-columns: 52px 48px minmax(0, 1fr);
  align-items: baseline;
  gap: 6px;
}

.quota-tooltip__row span,
.quota-tooltip__row small,
.quota-tooltip > small {
  color: var(--st-text-secondary);
  font-size: 11px;
}

.quota-tooltip__row strong {
  color: var(--st-text-primary);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}
</style>
