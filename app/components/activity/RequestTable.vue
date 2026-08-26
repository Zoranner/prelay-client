<script setup lang="ts">
import type { RequestLog } from "~/stores/relay";
import {
  Badge,
  Button,
  Icon,
  Modal,
  RadioGroup,
  Select,
  Table,
  Tag,
} from "@stellar/ui";
import {
  requestDiagnostics,
  type RequestDiagnostics,
} from "~/utils/diagnosticMetadata";
import { protocolLabel, protocolTagVariant } from "~/utils/providerTemplates";

type RequestTableRow = RequestLog &
  Record<string, unknown> & {
    diagnostics: RequestDiagnostics | null;
  };

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
  { key: "model_requested", title: "请求模型", ellipsis: true },
  { key: "upstream", title: "供应商 / 上游模型" },
  { key: "protocol_in", title: "协议" },
  { key: "mode", title: "模式", width: 80, ellipsis: true },
  { key: "status", title: "状态", width: 112, ellipsis: true },
  { key: "input", title: "输入" },
  { key: "output", title: "输出" },
  { key: "latency", title: "耗时" },
];
const diagnosticColumns = [
  { key: "diagnostic", title: "诊断", width: 340, ellipsis: true },
  { key: "severity", title: "级别", width: 88 },
  { key: "count", title: "次数", width: 72 },
  { key: "paths", title: "路径样例", width: 320, ellipsis: true },
];
const visibleRequests = computed(() =>
  statusFilter.value === "all"
    ? props.requests
    : props.requests.filter((row) => row.status === statusFilter.value),
);
const tableRows = computed<RequestTableRow[]>(
  () =>
    visibleRequests.value.map((request) => ({
      ...request,
      model_upstream: request.model_upstream ?? "-",
      diagnostics: requestDiagnostics(request.metadata_json),
    })),
);
const selectedDiagnostics = ref<RequestDiagnostics | null>(null);
const workspaceExit = useWorkspaceExitGuard();
let diagnosticsExitRegistration:
  | ReturnType<typeof workspaceExit.register>
  | undefined;
const diagnosticsDialogOpen = computed({
  get: () => selectedDiagnostics.value !== null,
  set: (visible: boolean) => {
    if (!visible) selectedDiagnostics.value = null;
  },
});
const diagnosticRows = computed(() =>
  selectedDiagnostics.value?.diagnostics.map((diagnostic) => ({
    ...diagnostic,
    diagnostic: `${diagnostic.code}\n${diagnostic.message}`,
    paths: diagnostic.paths.join("\n") || "-",
  })) ?? [],
);

function statusTitle(row: RequestLog) {
  return (
    [row.error_code, row.error_message].filter(Boolean).join("\n") || undefined
  );
}

function upstreamTitle(row: RequestTableRow) {
  return (
    [row.provider_name, row.model_upstream].filter(Boolean).join("\n") || "-"
  );
}

function formatMetric(value: number | null, unit = "") {
  return value === null ? "-" : `${value.toLocaleString()}${unit}`;
}

function openDiagnostics(diagnostics: RequestDiagnostics) {
  selectedDiagnostics.value = diagnostics;
}

watch(diagnosticsDialogOpen, (isOpen) => {
  if (!isOpen) {
    diagnosticsExitRegistration?.unregister();
    diagnosticsExitRegistration = undefined;
    return;
  }
  diagnosticsExitRegistration = workspaceExit.register({
    close: () => {
      selectedDiagnostics.value = null;
    },
    state: () => "allow",
  });
});

onBeforeUnmount(() => diagnosticsExitRegistration?.unregister());

function updateLimit(value: string | number | boolean | null) {
  if (typeof value === "number") {
    emit("update:limit", value);
    emit("reload");
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
          variant="button"
        />
        <Select
          class="activity-toolbar__limit"
          aria-label="显示条数"
          :model-value="limit"
          :options="limitOptions"
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
        <template #cell-upstream="{ row }">
          <div class="activity-provider-model" :title="upstreamTitle(row)">
            <span class="activity-provider-model__provider">{{
              row.provider_name ?? "-"
            }}</span>
            <span class="activity-provider-model__model">{{
              row.model_upstream
            }}</span>
          </div>
        </template>
        <template #cell-protocol_in="{ row }">
          <Tag
            size="small"
            :title="protocolLabel(row.protocol_in)"
            :variant="protocolTagVariant(row.protocol_in)"
          >
            {{ protocolLabel(row.protocol_in) }}
          </Tag>
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
          <div class="activity-status">
            <Badge
              :variant="row.status === 'failed' ? 'danger' : 'success'"
              :title="statusTitle(row)"
            >
              {{ row.http_status ?? row.status }}
            </Badge>
            <Button
              v-if="row.diagnostics"
              square
              size="tiny"
              variant="text"
              icon="ph:warning-circle"
              aria-label="查看请求诊断"
              title="查看请求诊断"
              @click="openDiagnostics(row.diagnostics)"
            />
          </div>
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
        <template #cell-created_at="{ row }">
          {{ new Date(row.created_at).toLocaleString() }}
        </template>
      </Table>
    </div>
  </div>

  <Modal
    v-model:visible="diagnosticsDialogOpen"
    title="请求诊断"
    size="xlarge"
    height="min(680px, calc(100dvh - 4rem))"
    :show-cancel="false"
    :show-confirm="false"
  >
    <div v-if="selectedDiagnostics" class="diagnostics-detail">
      <div
        v-if="selectedDiagnostics.streamIssue"
        class="diagnostics-stream-issue"
      >
        <Icon icon="ph:warning" />
        {{ selectedDiagnostics.streamIssue }}
      </div>
      <Table
        v-if="diagnosticRows.length"
        class="diagnostics-detail__table"
        :columns="diagnosticColumns"
        :data="diagnosticRows"
        fixed-header
        layout="fixed"
        row-key="code"
      >
        <template #cell-diagnostic="{ row }">
          <div class="diagnostics-entry" :title="row.diagnostic">
            <span class="diagnostics-entry__code">{{ row.code }}</span>
            <span class="diagnostics-entry__message">{{ row.message }}</span>
          </div>
        </template>
        <template #cell-severity="{ row }">
          <Badge :variant="row.severity === 'warning' ? 'warning' : 'default'">
            {{ row.severity === "warning" ? "警告" : "提示" }}
          </Badge>
        </template>
        <template #cell-paths="{ row }">
          <span class="diagnostics-paths" :title="row.paths">{{
            row.paths
          }}</span>
        </template>
      </Table>
    </div>
    <template #footer>
      <Button @click="diagnosticsDialogOpen = false">关闭</Button>
    </template>
  </Modal>
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

.activity-status {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.diagnostics-detail {
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  padding: var(--spacing-lg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.diagnostics-detail__table {
  width: 100%;
  min-height: 0;
  flex: 1;
}

.diagnostics-stream-issue {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-md);
  color: var(--st-warning);
}

.diagnostics-entry {
  display: grid;
  gap: var(--spacing-2xs);
  min-width: 0;
}

.diagnostics-entry__code,
.diagnostics-entry__message,
.diagnostics-paths {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diagnostics-entry__code {
  color: var(--st-text-primary);
}

.diagnostics-entry__message,
.diagnostics-paths {
  color: var(--st-text-muted);
  font-size: 12px;
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
