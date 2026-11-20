<script setup lang="ts">
defineOptions({ name: "ProviderQuotaCell" });

import { Icon, ProgressBar, Spinner, Tooltip } from "@stellar/ui";
import type { ProviderQuotaResponse, QuotaWindow } from "~/types/quota";
import { formatTokens } from "~/utils/stats/tokenFormat";
import QuotaPlanRings from "./PlanRings.vue";

const props = defineProps<{
  quota: ProviderQuotaResponse | null;
  loading?: boolean;
  providerName?: string;
}>();

const entry = computed(() => props.quota?.quota ?? null);
const windows = computed<QuotaWindow[]>(() =>
  (entry.value?.windows ?? []).filter((window) => !window.aside),
);
const balance = computed(() => entry.value?.balance ?? null);
const error = computed(() => entry.value?.error ?? null);
const stale = computed(() => entry.value?.stale === true);

function clampPercent(value: number) {
  return Math.max(0, Math.min(100, Number.isFinite(value) ? value : 0));
}

function remainingPercent(window: QuotaWindow) {
  return clampPercent(100 - window.used_percent);
}

function unitOrder(window: QuotaWindow) {
  if (window.span_secs != null && window.span_secs > 0) {
    return window.span_secs;
  }
  const label = window.label.replace(/\s+/g, "");
  if (label.includes("小时")) return 5 * 60 * 60;
  if (label === "日") return 24 * 60 * 60;
  if (label.includes("周") || label.includes("7天")) {
    return 7 * 24 * 60 * 60;
  }
  if (label.includes("月")) return 31 * 24 * 60 * 60;
  return Number.MAX_SAFE_INTEGER;
}

const tightest = computed(() => {
  if (!windows.value.length) return 100;
  return Math.min(...windows.value.map(remainingPercent));
});

const tightestWindow = computed(() => {
  const remaining = tightest.value;
  return windows.value
    .filter((window) => Math.abs(remainingPercent(window) - remaining) < 0.5)
    .sort((left, right) => unitOrder(left) - unitOrder(right))[0];
});

const tightestLabel = computed(() => tightestWindow.value?.label ?? "");

const valueTone = computed(() => {
  if (tightest.value <= 15) return "danger" as const;
  if (tightest.value <= 35) return "warning" as const;
  return "primary" as const;
});

const statusTone = computed(() => {
  const code = error.value?.code;
  return code && code !== "unsupported" && code !== "unconfigured"
    ? "danger"
    : "muted";
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

const profileLabel = computed(() => {
  const mode = props.quota?.mode;
  const parts = [
    mode === "shared_limited"
      ? "共享限额"
      : mode === "shared_unlimited"
        ? "共享"
        : "个人",
  ];
  if (entry.value?.plan) parts.push(planLabel(entry.value.plan));
  else if (props.quota?.owner_display_name) {
    parts.push(props.quota.owner_display_name);
  }
  return parts.join(" · ");
});

const tooltipStatus = computed(() => {
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
  return stale.value ? "旧读数" : "正常";
});

const tooltipStatusTone = computed(() => {
  if (error.value) {
    return error.value.code === "unsupported" ||
      error.value.code === "unconfigured"
      ? "muted"
      : "danger";
  }
  return stale.value ? "muted" : "success";
});

const updatedLabel = computed(() => {
  const value = props.quota?.updated_at;
  if (!value) return "更新时间未知";
  return `更新 ${new Date(value).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  })}`;
});

function formatPercent(value: number) {
  const percent = clampPercent(value);
  return `${percent >= 99.5 ? 100 : Math.round(percent)}%`;
}

function planLabel(plan: string) {
  const id = plan.toLowerCase();
  if (id.includes("goat")) return "GOAT";
  if (id.includes("ultra")) return "Ultra";
  if (id.includes("max-20x")) return "Max 20×";
  if (id.includes("max-10x")) return "Max 10×";
  if (id.includes("individual-max")) return "Max";
  if (id.includes("individual-pro")) return "Pro";
  if (id.includes("individual-go")) return "Go";
  if (id.includes("teams-pro") || id.includes("team-pro")) return "Team Pro";
  return plan;
}

function progressSemantic(window: QuotaWindow) {
  const remaining = remainingPercent(window);
  if (remaining <= 15) return "error" as const;
  if (remaining <= 35) return "warning" as const;
  return "primary" as const;
}

function percentTone(window: QuotaWindow) {
  const remaining = remainingPercent(window);
  if (remaining <= 15) return "danger" as const;
  if (remaining <= 35) return "warning" as const;
  return "primary" as const;
}

function formatQuotaAmount(window: QuotaWindow) {
  const amount = Number(window.amount);
  const limit = window.limit == null ? Number.NaN : Number(window.limit);
  if (Number.isFinite(amount) && window.unit === "USD") {
    const remaining = `$${amount.toFixed(2)}`;
    return Number.isFinite(limit)
      ? `${remaining} / $${limit.toFixed(2)}`
      : remaining;
  }
  if (Number.isFinite(amount) && (!window.unit || window.unit === "tokens")) {
    return formatTokens(amount);
  }
  return window.display ?? window.amount ?? "";
}

function formatCountdown(value: string | null | undefined) {
  if (!value) return "重置未知";
  const diff = new Date(value).getTime() - Date.now();
  if (!Number.isFinite(diff) || diff <= 0) return "即将重置";
  const minutes = Math.ceil(diff / 60_000);
  if (minutes < 60) return `${minutes} 分钟后重置`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    const rest = minutes % 60;
    return rest ? `${hours}h ${rest}m 后重置` : `${hours}h 后重置`;
  }
  const days = Math.floor(hours / 24);
  const rest = hours % 24;
  return rest ? `${days}d ${rest}h 后重置` : `${days}d 后重置`;
}

function quotaWindowMeta(window: QuotaWindow) {
  const parts: string[] = [];
  if (window.amount != null) {
    parts.push(`剩 ${formatQuotaAmount(window)}`);
  } else if (window.display) {
    parts.push(window.display);
  }
  parts.push(window.resets_at ? formatCountdown(window.resets_at) : "重置未知");
  if (window.pool) parts.push(window.pool);
  return parts.join(" · ");
}
</script>

<template>
  <Tooltip v-if="quota">
    <div class="quota-cell">
      <span class="quota-cell__lead">
        <QuotaPlanRings v-if="windows.length" :windows="windows" />
        <span v-else-if="balance" class="quota-cell__icon-frame">
          <Icon class="quota-cell__icon" name="ph:wallet" />
        </span>
        <span
          v-else
          class="quota-cell__icon-frame"
          :class="{ 'quota-cell__icon-frame--danger': statusTone === 'danger' }"
        >
          <Icon
            class="quota-cell__icon"
            :name="statusTone === 'danger' ? 'ph:warning-circle' : 'ph:plugs'"
          />
        </span>
      </span>

      <template v-if="windows.length">
        <span class="quota-cell__main">
          <span class="quota-cell__window">{{ tightestLabel }}</span>
          <span
            class="quota-cell__value"
            :class="{
              'quota-cell__value--warning': valueTone === 'warning',
              'quota-cell__value--low': valueTone === 'danger',
            }"
          >
            {{ formatPercent(tightest) }}
          </span>
        </span>
        <span v-if="balance" class="quota-cell__balance">
          余额 {{ balance.text }}
        </span>
      </template>
      <span v-else-if="balance" class="quota-cell__main">
        <span class="quota-cell__label">余额</span>
        <span
          class="quota-cell__amount"
          :class="{ 'quota-cell__amount--low': !balance.available }"
        >
          {{ balance.text }}
        </span>
      </span>
      <span
        v-else
        class="quota-cell__status quota-cell__status--chip"
        :class="{ 'quota-cell__status--danger': statusTone === 'danger' }"
      >
        {{ statusText }}
      </span>

      <span v-if="stale" class="quota-cell__stale">旧</span>
    </div>

    <template #content>
      <div class="quota-tooltip">
        <div class="quota-tooltip__head">
          <div class="quota-tooltip__title">
            <strong>{{ providerName || "供应商" }}</strong>
            <span>{{ profileLabel }}</span>
          </div>
          <span
            v-if="tooltipStatusTone !== 'success'"
            class="quota-tooltip__status"
            :class="`quota-tooltip__status--${tooltipStatusTone}`"
          >
            {{ tooltipStatus }}
          </span>
        </div>

        <div v-if="windows.length" class="quota-tooltip__list">
          <div
            v-for="window in windows"
            :key="window.label"
            class="quota-tooltip__row"
          >
            <div class="quota-tooltip__line">
              <span>{{ window.label }}</span>
              <strong
                :class="{
                  'quota-tooltip__percent--warning':
                    percentTone(window) === 'warning',
                  'quota-tooltip__percent--danger':
                    percentTone(window) === 'danger',
                }"
              >
                {{ formatPercent(remainingPercent(window)) }}
              </strong>
            </div>
            <div class="quota-tooltip__bar">
              <ProgressBar
                :value="remainingPercent(window)"
                :max="100"
                :show-header="false"
                size="small"
                :semantic="progressSemantic(window)"
              />
            </div>
            <small>{{ quotaWindowMeta(window) }}</small>
          </div>
        </div>

        <p
          v-if="error"
          class="quota-tooltip__message"
          :class="{
            'quota-tooltip__message--danger': tooltipStatusTone === 'danger',
          }"
        >
          {{ error.message }}
        </p>
        <p
          v-else-if="!windows.length && !balance"
          class="quota-tooltip__message"
        >
          暂无额度读数
        </p>
        <p v-if="stale" class="quota-tooltip__message">显示的是上次成功读数</p>

        <div class="quota-tooltip__foot">
          <span v-if="balance">余额 {{ balance.text }}</span>
          <span v-else-if="windows.length">无余额</span>
          <span>{{ updatedLabel }}</span>
        </div>
      </div>
    </template>
  </Tooltip>
  <span
    v-else-if="loading"
    class="quota-cell"
    role="status"
    :aria-label="statusText"
  >
    <span class="quota-cell__lead">
      <Spinner size="sm" />
    </span>
    <span class="quota-cell__skeleton" aria-hidden="true"></span>
  </span>
  <span v-else class="quota-cell__status quota-cell__status--muted">
    {{ statusText }}
  </span>
</template>

<style scoped src="./QuotaCell.css"></style>
