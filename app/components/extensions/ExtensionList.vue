<script setup lang="ts">
import { Badge, Button, Table, useNotification } from "stellar-ui";
import type { AgentExtension } from "~/stores/relay";

type ExtensionRow = AgentExtension & { id: string } & Record<string, unknown>;

const props = defineProps<{
  extensions: AgentExtension[];
}>();
const notifications = useNotification();

const columns = [
  { key: "name", title: "名称", width: 180, ellipsis: true },
  { key: "version", title: "版本", width: 120, ellipsis: true },
  { key: "status", title: "状态", width: 88 },
  { key: "sourcePath", title: "来源", ellipsis: true },
];

const rows = computed<ExtensionRow[]>(() =>
  props.extensions.map((extension) => ({
    ...extension,
    id: `${extension.kind}-${extension.name}-${extension.sourcePath}`,
  })),
);
function statusLabel(status: AgentExtension["status"]) {
  return { enabled: "启用", disabled: "禁用", error: "错误" }[status];
}

function statusVariant(
  status: AgentExtension["status"],
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
    class="extension-table"
    :columns="columns"
    :data="rows"
    empty-text="未发现扩展"
    fixed-header
    layout="fixed"
    row-key="id"
  >
    <template #cell-name="{ row }">
      <div class="extension-name">
        <span>{{ row.name }}</span>
        <small v-if="row.errorMessage">{{ row.errorMessage }}</small>
      </div>
    </template>
    <template #cell-version="{ row }">
      {{ row.version || "-" }}
    </template>
    <template #cell-sourcePath="{ row }">
      <div class="extension-source">
        <span :title="row.sourcePath">{{ row.sourcePath }}</span>
        <Button
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
  </Table>
</template>

<style scoped>
.extension-table {
  min-height: 0;
  flex: 1;
}

.extension-table :deep(.relative) {
  overflow-x: hidden;
}

.extension-name {
  display: grid;
  gap: 2px;
}

.extension-source {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-xs);
}

.extension-source > span {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.extension-name > small {
  color: var(--st-text-danger);
  font-size: 12px;
}

</style>
