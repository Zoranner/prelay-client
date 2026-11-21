<script setup lang="ts">
defineOptions({ name: "ProviderSharedQuotaCell" });

import { ProgressBar, Spinner, Tooltip } from "@stellar/ui";
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
const sharedView = computed(() =>
  view.value?.variant === "shared" ? view.value : null,
);

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
  <Tooltip v-if="quota && sharedView && sharedView.primary.value">
    <div class="quota-cell">
      <span class="quota-cell__lead">
        <QuotaRings :slots="sharedView.slots" />
      </span>

      <span class="quota-cell__main">
        <span class="quota-cell__window">{{ sharedView.primary.label }}</span>
        <span
          class="quota-cell__value"
          :class="{
            'quota-cell__value--warning': sharedView.primary.tone === 'warning',
            'quota-cell__value--low': sharedView.primary.tone === 'danger',
          }"
        >
          {{ sharedView.primary.value }}
        </span>
      </span>

      <span v-if="sharedView.stale" class="quota-cell__stale">旧</span>
    </div>

    <template #content>
      <div class="quota-tooltip">
        <div class="quota-tooltip__head">
          <div class="quota-tooltip__title">
            <strong>{{ providerName || "供应商" }}</strong>
            <span>{{ sharedView.tooltip.profile }}</span>
          </div>
          <span
            v-if="sharedView.tooltip.statusTone !== 'success'"
            class="quota-tooltip__status"
            :class="`quota-tooltip__status--${sharedView.tooltip.statusTone}`"
          >
            {{ sharedView.tooltip.status }}
          </span>
        </div>

        <div v-if="sharedView.tooltip.rows.length" class="quota-tooltip__list">
          <div
            v-for="row in sharedView.tooltip.rows"
            :key="row.key"
            class="quota-tooltip__row"
          >
            <div class="quota-tooltip__line">
              <span>{{ row.label }}</span>
              <strong :class="percentClass(row.tone)"
                >剩 {{ row.value }}</strong
              >
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

        <p v-if="sharedView.tooltip.message" class="quota-tooltip__message">
          {{ sharedView.tooltip.message }}
        </p>
        <p v-if="sharedView.stale" class="quota-tooltip__message">
          显示的是上次成功读数
        </p>

        <div class="quota-tooltip__foot">
          <span v-for="item in sharedView.tooltip.footer" :key="item">
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
    aria-label="查询中"
  >
    <span class="quota-cell__lead">
      <Spinner size="sm" />
    </span>
    <span class="quota-cell__skeleton" aria-hidden="true"></span>
  </span>
  <span v-else class="quota-cell__status quota-cell__status--chip">
    {{ sharedView ? "未配置" : "—" }}
  </span>
</template>

<style scoped src="./QuotaCell.css"></style>
