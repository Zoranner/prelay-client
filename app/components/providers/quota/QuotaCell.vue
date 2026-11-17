<script setup lang="ts">
defineOptions({ name: "ProviderQuotaCell" });

import { ProgressBar, Tooltip } from "@stellar/ui";
import type {
  ProviderQuotaResponse,
  QuotaWindow,
  QuotaWindowKind,
} from "~/types/quota";
import { formatTokens } from "~/utils/stats/tokenFormat";
import QuotaPlanRings from "./PlanRings.vue";

const props = defineProps<{
  quota: ProviderQuotaResponse | null;
  loading?: boolean;
}>();

const windows = computed<QuotaWindow[]>(() => {
  const quota = props.quota?.quota;
  if (!quota) return [];
  return quota.kind === "plan" || quota.kind === "shared" ? quota.windows : [];
});

const tightest = computed(() =>
  Math.min(
    100,
    ...windows.value.map((window) =>
      Math.max(0, Math.min(100, window.remaining_percent)),
    ),
  ),
);

const progressSemantic = computed(() => {
  if (tightest.value <= 15) return "error" as const;
  if (tightest.value <= 35) return "warning" as const;
  return "success" as const;
});

const balance = computed(() =>
  props.quota?.quota.kind === "balance" ? props.quota.quota : null,
);

const statusText = computed(() => {
  if (props.loading && !props.quota) return "查询中";
  if (!props.quota) return "—";
  switch (props.quota.quota.kind) {
    case "unsupported":
      return "未接入";
    case "error":
      return "查询失败";
    default:
      return "";
  }
});

const windowLabels: Record<QuotaWindowKind, string> = {
  five_hour: "5 小时",
  day: "日",
  week: "周",
  month: "月",
};

function formatPercent(value: number) {
  const percent = Math.max(0, Math.min(100, value));
  return `${percent >= 99.5 ? 100 : Math.round(percent)}%`;
}

function formatReset(value: string | null) {
  if (!value) return "未知";
  return new Date(value).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatAmount(amount: string, currency: string) {
  const symbol = currency === "CNY" ? "¥" : currency === "USD" ? "$" : "";
  return `${symbol}${amount}${symbol ? "" : ` ${currency}`}`;
}
</script>

<template>
  <Tooltip
    v-if="
      quota &&
      (quota.quota.kind === 'plan' ||
        quota.quota.kind === 'shared' ||
        quota.quota.kind === 'balance')
    "
  >
    <div class="quota-cell">
      <QuotaPlanRings v-if="quota.quota.kind === 'plan'" :windows="windows" />
      <ProgressBar
        v-else-if="quota.quota.kind === 'shared'"
        class="quota-cell__bar"
        :value="tightest"
        :max="100"
        :show-header="false"
        size="small"
        :semantic="progressSemantic"
      />
      <span
        v-if="quota.quota.kind === 'plan' || quota.quota.kind === 'shared'"
        class="quota-cell__value"
        :class="{ 'quota-cell__value--low': tightest <= 15 }"
      >
        {{ formatPercent(tightest) }}
      </span>
      <span
        v-else-if="balance"
        class="quota-cell__balance"
        :class="{ 'quota-cell__balance--low': !balance.available }"
      >
        {{ formatAmount(balance.amount, balance.currency) }}
      </span>
    </div>

    <template #content>
      <div class="quota-tooltip">
        <template v-if="windows.length">
          <div
            v-for="window in windows"
            :key="window.window"
            class="quota-tooltip__row"
          >
            <span>{{ windowLabels[window.window] }}</span>
            <strong>{{ formatPercent(window.remaining_percent) }}</strong>
            <small>
              <template v-if="window.remaining != null">
                剩 {{ formatTokens(window.remaining) }} ·
              </template>
              重置 {{ formatReset(window.resets_at)
              }}<template v-if="window.accumulate"> · 累计</template>
            </small>
          </div>
        </template>
        <template v-else-if="balance">
          <div class="quota-tooltip__row">
            <span>可用余额</span>
            <strong>{{
              formatAmount(balance.amount, balance.currency)
            }}</strong>
          </div>
          <small>
            {{ balance.available ? "账户可用" : "余额不足或不可用" }}
          </small>
        </template>
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
    :title="quota?.quota.kind === 'error' ? quota.quota.message : undefined"
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

.quota-cell__value--low,
.quota-cell__balance--low {
  color: var(--st-danger);
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
  min-width: 190px;
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
