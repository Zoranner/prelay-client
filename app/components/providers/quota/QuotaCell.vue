<script setup lang="ts">
defineOptions({ name: "ProviderQuotaCell" });

import { Icon, ProgressBar, Spinner, Tooltip } from "@stellar/ui";
import type { ProviderQuotaResponse } from "~/types/quota";
import { buildQuotaView } from "~/utils/provider/quota";
import type { QuotaTone } from "~/utils/provider/quota";
import QuotaRings from "./QuotaRings.vue";

const props = defineProps<{
  quota: ProviderQuotaResponse | null;
  loading?: boolean;
  providerName?: string;
}>();

const view = computed(() => (props.quota ? buildQuotaView(props.quota) : null));
const planView = computed(() =>
  view.value?.variant === "plan" ? view.value : null,
);
const balanceView = computed(() =>
  view.value?.variant === "balance" ? view.value : null,
);
const statusView = computed(() =>
  view.value?.variant === "status" ? view.value : null,
);
const contentView = computed(() => planView.value ?? balanceView.value);
const statusText = computed(() => {
  if (props.loading && !props.quota) return "查询中";
  if (!props.quota) return "—";
  return statusView.value?.primary.value ?? "—";
});
const statusTone = computed(() => statusView.value?.primary.tone ?? "muted");

function valueClass(tone: QuotaTone) {
  return {
    "quota-cell__value--warning": tone === "warning",
    "quota-cell__value--low": tone === "danger",
  };
}

function percentClass(tone: QuotaTone) {
  return {
    "quota-tooltip__percent--warning": tone === "warning",
    "quota-tooltip__percent--danger": tone === "danger",
  };
}

function progressSemantic(tone: QuotaTone) {
  if (tone === "danger") return "error" as const;
  if (tone === "warning") return "warning" as const;
  return "primary" as const;
}
</script>

<template>
  <Tooltip v-if="quota && contentView">
    <div class="quota-cell">
      <span class="quota-cell__lead">
        <QuotaRings v-if="planView" :slots="planView.slots" />
        <span v-else class="quota-cell__icon-frame">
          <Icon class="quota-cell__icon" name="ph:wallet" />
        </span>
      </span>

      <template v-if="planView">
        <span class="quota-cell__main">
          <span class="quota-cell__window">{{ planView.primary.label }}</span>
          <span
            class="quota-cell__value"
            :class="valueClass(planView.primary.tone)"
          >
            {{ planView.primary.value }}
          </span>
        </span>
        <span v-if="planView.secondary" class="quota-cell__balance">
          {{ planView.secondary.label }} {{ planView.secondary.value }}
        </span>
      </template>
      <span v-else-if="balanceView" class="quota-cell__main">
        <span class="quota-cell__label">{{ balanceView.primary.label }}</span>
        <span
          class="quota-cell__amount"
          :class="valueClass(balanceView.primary.tone)"
        >
          {{ balanceView.primary.value }}
        </span>
      </span>

      <span v-if="contentView.stale" class="quota-cell__stale">旧</span>
    </div>

    <template #content>
      <div class="quota-tooltip">
        <div class="quota-tooltip__head">
          <div class="quota-tooltip__title">
            <strong>{{ providerName || "供应商" }}</strong>
            <span>{{ contentView.tooltip.profile }}</span>
          </div>
          <span
            v-if="contentView.tooltip.statusTone !== 'success'"
            class="quota-tooltip__status"
            :class="`quota-tooltip__status--${contentView.tooltip.statusTone}`"
          >
            {{ contentView.tooltip.status }}
          </span>
        </div>

        <div v-if="contentView.tooltip.rows.length" class="quota-tooltip__list">
          <div
            v-for="row in contentView.tooltip.rows"
            :key="row.key"
            class="quota-tooltip__row"
          >
            <div class="quota-tooltip__line">
              <span>{{ row.label }}</span>
              <strong :class="percentClass(row.tone)">{{ row.value }}</strong>
            </div>
            <div class="quota-tooltip__bar">
              <ProgressBar
                :value="row.percent"
                :max="100"
                :show-header="false"
                size="small"
                :semantic="progressSemantic(row.tone)"
              />
            </div>
            <small>{{ row.meta }}</small>
          </div>
        </div>

        <p
          v-if="contentView.tooltip.message"
          class="quota-tooltip__message"
          :class="{
            'quota-tooltip__message--danger':
              contentView.tooltip.statusTone === 'danger',
          }"
        >
          {{ contentView.tooltip.message }}
        </p>
        <p
          v-else-if="!contentView.tooltip.rows.length"
          class="quota-tooltip__message"
        >
          暂无额度读数
        </p>
        <p v-if="contentView.stale" class="quota-tooltip__message">
          显示的是上次成功读数
        </p>

        <div class="quota-tooltip__foot">
          <span v-for="item in contentView.tooltip.footer" :key="item">
            {{ item }}
          </span>
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
  <span
    v-else
    class="quota-cell__status quota-cell__status--chip"
    :class="{ 'quota-cell__status--danger': statusTone === 'danger' }"
  >
    {{ statusText }}
  </span>
</template>

<style scoped src="./QuotaCell.css"></style>
