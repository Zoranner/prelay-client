<script setup lang="ts">
import type { Activity } from "~/stores/relay";
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
import { protocolLabel, protocolTagVariant } from "~/utils/providerTemplates";

type RequestTableRow = Activity & Record<string, unknown>;

const props = defineProps<{
  limit: number;
  pending?: boolean;
  requests: Activity[];
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
const visibleRequests = computed(() =>
  statusFilter.value === "all"
    ? props.requests
    : props.requests.filter((row) => row.status === statusFilter.value),
);
const tableRows = computed<RequestTableRow[]>(() =>
  visibleRequests.value.map((request) => ({
    ...request,
    model_upstream: request.model_upstream ?? "-",
  })),
);
const selectedError = ref<Activity | null>(null);
const workspaceExit = useWorkspaceExitGuard();
let errorExitRegistration:
  ReturnType<typeof workspaceExit.register> | undefined;
const errorDialogOpen = computed({
  get: () => selectedError.value !== null,
  set: (visible: boolean) => {
    if (!visible) selectedError.value = null;
  },
});

function upstreamTitle(row: RequestTableRow) {
  return (
    [row.provider_name, row.model_upstream].filter(Boolean).join("\n") || "-"
  );
}

function formatMetric(value: number | null, unit = "") {
  return value === null ? "-" : `${value.toLocaleString()}${unit}`;
}

function openError(activity: Activity) {
  selectedError.value = activity;
}

watch(errorDialogOpen, (isOpen) => {
  if (!isOpen) {
    errorExitRegistration?.unregister();
    errorExitRegistration = undefined;
    return;
  }
  errorExitRegistration = workspaceExit.register({
    close: () => {
      selectedError.value = null;
    },
    state: () => "allow",
  });
});

onBeforeUnmount(() => errorExitRegistration?.unregister());

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
        empty-text="暂无活动"
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
            >
              {{ row.http_status ?? row.status }}
            </Badge>
            <Button
              v-if="row.status === 'failed'"
              square
              size="tiny"
              variant="text"
              icon="ph:info"
              aria-label="查看活动错误"
              title="查看活动错误"
              @click="openError(row)"
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
    v-model:visible="errorDialogOpen"
    title="活动错误"
    size="large"
    :show-cancel="false"
    :show-confirm="false"
  >
    <div v-if="selectedError" class="activity-error-detail">
      <dl>
        <div>
          <dt>HTTP 状态</dt>
          <dd>{{ selectedError.http_status ?? "-" }}</dd>
        </div>
        <div>
          <dt>错误码</dt>
          <dd>{{ selectedError.error_code ?? "-" }}</dd>
        </div>
        <div>
          <dt>错误说明</dt>
          <dd class="activity-error-detail__message">
            {{ selectedError.error_message ?? "上游未提供错误说明" }}
          </dd>
        </div>
      </dl>
    </div>
    <template #footer>
      <Button @click="errorDialogOpen = false">关闭</Button>
    </template>
  </Modal>
</template>

<style scoped src="./RequestTable.css"></style>
