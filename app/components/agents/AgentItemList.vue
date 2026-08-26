<script setup lang="ts">
import { Badge, Button, Table, useNotification } from "@stellar/ui";
import type { AgentItem } from "~/stores/relay";

type AgentItemRow = AgentItem & { id: string } & Record<string, unknown>;

const props = defineProps<{
  items: AgentItem[];
  pending?: boolean;
}>();
const emit = defineEmits<{
  uninstall: [item: AgentItem];
}>();
const notifications = useNotification();

const columns = [
  { key: "name", title: "名称", width: 180, ellipsis: true },
  { key: "version", title: "版本", width: 120, ellipsis: true },
  { key: "status", title: "状态", width: 88 },
  { key: "sourcePath", title: "来源", minWidth: 360, ellipsis: true },
  {
    key: "actions",
    title: "操作",
    width: 64,
    align: "right" as const,
    fixed: "right" as const,
  },
];

const rows = computed<AgentItemRow[]>(() =>
  props.items.map((item) => ({
    ...item,
    id: `${item.kind}-${item.name}-${item.sourcePath}`,
  })),
);
function statusLabel(status: AgentItem["status"]) {
  return { enabled: "启用", disabled: "禁用", error: "错误" }[status];
}

function statusVariant(
  status: AgentItem["status"],
): "success" | "default" | "danger" {
  switch (status) {
    case "enabled":
      return "success";
    case "error":
      return "danger";
    default:
      return "default";
  }
}

async function copySourcePath(sourcePath: string) {
  try {
    await navigator.clipboard.writeText(sourcePath);
    notifications.success("已复制来源路径");
  } catch {
    notifications.danger("请手动复制。", { title: "无法访问剪贴板" });
  }
}
</script>

<template>
  <Table
    class="agent-item-table"
    :columns="columns"
    :data="rows"
    empty-text="未发现条目"
    fixed-header
    layout="fixed"
    row-key="id"
  >
    <template #cell-name="{ row }">
      <div class="agent-item-name">
        <span>{{ row.name }}</span>
        <small v-if="row.errorMessage">{{ row.errorMessage }}</small>
      </div>
    </template>
    <template #cell-version="{ row }">
      {{ row.version || "-" }}
    </template>
    <template #cell-sourcePath="{ row }">
      <div class="agent-item-source">
        <span :title="row.sourcePath">{{ row.sourcePath }}</span>
        <Button
          class="agent-item-source__copy"
          square
          size="small"
          variant="ghost"
          icon="ph:copy"
          aria-label="复制来源路径"
          title="复制来源路径"
          @click.stop="copySourcePath(row.sourcePath)"
        />
      </div>
    </template>
    <template #cell-status="{ row }">
      <Badge :variant="statusVariant(row.status)">
        {{ statusLabel(row.status) }}
      </Badge>
    </template>
    <template #cell-actions="{ row }">
      <Button
        square
        size="small"
        variant="danger"
        icon="ph:trash"
        aria-label="卸载"
        title="卸载"
        :disabled="pending || row.status === 'error'"
        @click.stop="emit('uninstall', row)"
      />
    </template>
  </Table>
</template>

<style scoped>
.agent-item-table {
  min-height: 0;
  flex: 1;
}

.agent-item-table :deep(.relative) {
  overflow-x: auto;
}

.agent-item-table :deep(.inline-block) {
  display: block;
  width: 100%;
  max-width: 100%;
}

.agent-item-name {
  display: grid;
  gap: 2px;
}

.agent-item-source {
  display: flex;
  width: 100%;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-xs);
}

.agent-item-source > span {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agent-item-source__copy {
  flex: 0 0 auto;
}

.agent-item-name > small {
  color: var(--st-danger);
  font-size: 12px;
}

</style>
