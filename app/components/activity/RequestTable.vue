<script setup lang="ts">
import type { RequestLog } from "~/stores/relay";
import {
  Badge,
  Button,
  Drawer,
  Icon,
  RadioGroup,
  Select,
  Table,
} from "stellar-ui";
import { formatDiagnosticMetadata } from "~/utils/diagnosticMetadata";

type RequestTableRow = RequestLog & Record<string, unknown>;

const props = defineProps<{
  limit: number;
  pending?: boolean;
  requests: RequestLog[];
}>();
const emit = defineEmits<{
  reload: [];
  "update:limit": [value: number];
}>();

const statusFilter = ref<"all" | "success" | "failed">("all");
const limitOptions = [
  { label: "50 条", value: 50 },
  { label: "100 条", value: 100 },
  { label: "200 条", value: 200 },
];
const statusOptions = [
  { label: "全部", value: "all" },
  { label: "成功", value: "success" },
  { label: "失败", value: "failed" },
];
const requestColumns = [
  { key: "created_at", title: "时间", width: 176, ellipsis: true },
  { key: "endpoint_name", title: "接入点" },
  { key: "provider", title: "供应商 / 模型" },
  { key: "protocol_in", title: "协议" },
  { key: "mode", title: "模式", width: 80, ellipsis: true },
  { key: "status", title: "状态", width: 80, ellipsis: true },
  { key: "input", title: "输入" },
  { key: "output", title: "输出" },
  { key: "latency", title: "耗时" },
  {
    key: "actions",
    title: "操作",
    width: 64,
    align: "center" as const,
    fixed: "right" as const,
  },
];
const visibleRequests = computed(() =>
  statusFilter.value === "all"
    ? props.requests
    : props.requests.filter((row) => row.status === statusFilter.value),
);
const tableRows = computed<RequestTableRow[]>(
  () => visibleRequests.value as RequestTableRow[],
);
const selectedMetadata = ref<string | null>(null);
const metadataDrawerOpen = computed(() => selectedMetadata.value !== null);
const metadataDetail = computed(() =>
  formatDiagnosticMetadata(selectedMetadata.value),
);

function protocolLabel(protocol: string | null) {
  switch (protocol) {
    case "chat_completions":
    case "openai":
      return "Chat Completions";
    case "responses":
      return "Responses";
    case "anthropic_messages":
    case "anthropic":
      return "Anthropic Messages";
    default:
      return "-";
  }
}

function statusTitle(row: RequestLog) {
  return (
    [row.error_code, row.error_message].filter(Boolean).join("\n") || undefined
  );
}

function providerModelTitle(row: RequestLog) {
  return (
    [row.provider_name, row.model_requested].filter(Boolean).join("\n") || "-"
  );
}

function formatMetric(value: number | null, unit = "") {
  return value === null ? "-" : `${value.toLocaleString()}${unit}`;
}

function hasMetadata(metadata: string | null) {
  return Boolean(formatDiagnosticMetadata(metadata));
}

function openMetadata(metadata: string | null) {
  if (hasMetadata(metadata)) {
    selectedMetadata.value = metadata;
  }
}

function updateLimit(value: string | number | boolean | null) {
  if (typeof value === "number") {
    emit("update:limit", value);
    emit("reload");
  }
}

function closeMetadata(visible: boolean) {
  if (!visible) {
    selectedMetadata.value = null;
  }
}
</script>

<template>
  <div class="activity-table">
    <div class="activity-toolbar">
      <div class="activity-toolbar__controls">
        <RadioGroup
          v-model="statusFilter"
          aria-label="状态筛选"
          :options="statusOptions"
          size="small"
          variant="button"
        />
        <Select
          class="activity-toolbar__limit"
          aria-label="显示条数"
          :model-value="limit"
          :options="limitOptions"
          size="small"
          @update:model-value="updateLimit"
        />
      </div>

      <Table
        class="activity-table__grid"
        :columns="requestColumns"
        :data="tableRows"
        empty-text="暂无请求记录"
        :loading="pending"
        layout="auto"
        row-key="id"
        fixed-header
      >
        <template #cell-endpoint_name="{ row }">
          <span
            class="activity-content-nowrap"
            :title="row.endpoint_name ?? '-'"
            >{{ row.endpoint_name ?? "-" }}</span
          >
        </template>
        <template #cell-provider="{ row }">
          <div class="activity-provider-model" :title="providerModelTitle(row)">
            <span class="activity-provider-model__provider">{{
              row.provider_name ?? "-"
            }}</span>
            <span class="activity-provider-model__model">{{
              row.model_requested ?? "-"
            }}</span>
          </div>
        </template>
        <template #cell-protocol_in="{ row }">
          <span
            class="activity-content-nowrap"
            :title="protocolLabel(row.protocol_in)"
            >{{ protocolLabel(row.protocol_in) }}</span
          >
        </template>
        <template #cell-mode="{ row }">
          {{
            row.is_streaming === null
              ? "-"
              : row.is_streaming
                ? "流式"
                : "非流式"
          }}
        </template>
        <template #cell-status="{ row }">
          <Badge
            :variant="row.status === 'failed' ? 'danger' : 'success'"
            :title="statusTitle(row)"
          >
            {{ row.http_status ?? row.status }}
          </Badge>
        </template>
        <template #cell-input="{ row }">
          <div class="activity-metric">
            <span class="activity-metric__value">
              <Icon icon="ph:arrow-down" />
              {{ formatMetric(row.input_tokens) }}
            </span>
            <span class="activity-metric__detail">
              <Icon icon="ph:database" />
              缓存读 {{ formatMetric(row.cache_read_tokens) }}
            </span>
          </div>
        </template>
        <template #cell-output="{ row }">
          <div class="activity-metric">
            <span class="activity-metric__value">
              <Icon icon="ph:arrow-up" />
              {{ formatMetric(row.output_tokens) }}
            </span>
            <span class="activity-metric__detail">
              <Icon icon="ph:database" />
              缓存写 {{ formatMetric(row.cache_write_tokens) }}
            </span>
          </div>
        </template>
        <template #cell-latency="{ row }">
          <div class="activity-metric">
            <span class="activity-metric__value">
              <Icon icon="ph:timer" />
              首 {{ formatMetric(row.first_token_ms, " ms") }}
            </span>
            <span class="activity-metric__detail">
              <Icon icon="ph:clock" />
              总 {{ formatMetric(row.latency_ms, " ms") }}
            </span>
          </div>
        </template>
        <template #cell-actions="{ row }">
          <Button
            square
            size="small"
            icon="ph:brackets-curly"
            aria-label="查看元数据"
            title="查看元数据"
            :disabled="!hasMetadata(row.metadata_json)"
            @click="openMetadata(row.metadata_json)"
          />
        </template>
        <template #cell-created_at="{ row }">
          {{ new Date(row.created_at).toLocaleString() }}
        </template>
      </Table>
    </div>
  </div>

  <Drawer
    :visible="metadataDrawerOpen"
    title="请求元数据"
    :show-confirm="false"
    @update:visible="closeMetadata"
  >
    <pre
      v-if="metadataDetail"
      class="max-h-96 overflow-auto whitespace-pre-wrap break-all"
      >{{ metadataDetail }}</pre>
  </Drawer>
</template>

<style scoped>
.activity-table {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  gap: var(--spacing-md);
}

.activity-toolbar {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  gap: var(--spacing-md);
}

.activity-toolbar__controls {
  display: flex;
  min-height: 32px;
  align-items: center;
  gap: var(--spacing-md);
}

.activity-toolbar__limit {
  width: 104px;
  margin-left: auto;
}

.activity-table__grid {
  min-height: 0;
  flex: 1;
  overflow: auto;
}

.activity-provider-model {
  display: grid;
  grid-template-rows: repeat(2, 18px);
  line-height: 18px;
}

.activity-metric {
  display: grid;
  grid-template-rows: repeat(2, 18px);
  line-height: 18px;
}

.activity-provider-model__provider,
.activity-provider-model__model {
  white-space: nowrap;
}

.activity-metric__value,
.activity-metric__detail {
  white-space: nowrap;
}

.activity-content-nowrap {
  white-space: nowrap;
}

.activity-provider-model__provider,
.activity-metric__value {
  color: var(--st-text-primary);
}

.activity-provider-model__model,
.activity-metric__detail {
  color: var(--st-text-muted);
  font-size: 12px;
}

.activity-metric__value,
.activity-metric__detail {
  display: flex;
  align-items: center;
  gap: 4px;
}

.activity-metric :deep(svg) {
  width: 12px;
  height: 12px;
  flex: 0 0 auto;
}

@media (max-width: 560px) {
  .activity-toolbar__controls {
    flex-wrap: wrap;
  }

  .activity-toolbar__limit {
    margin-left: 0;
  }
}
</style>
