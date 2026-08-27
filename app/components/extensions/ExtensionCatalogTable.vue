<script setup lang="ts">
import { Button, Table } from "@stellar/ui";
import type { ExtensionPackage } from "~/stores/relay";

type ExtensionRow = ExtensionPackage & Record<string, unknown>;

defineProps<{
  packages: ExtensionPackage[];
  pending?: boolean;
}>();
const emit = defineEmits<{
  detail: [item: ExtensionPackage];
  install: [item: ExtensionPackage];
}>();

const columns = [
  { key: "name", title: "名称", width: 180, ellipsis: true },
  { key: "version", title: "版本", width: 112, ellipsis: true },
  { key: "summary", title: "摘要", minWidth: 260, ellipsis: true },
  {
    key: "actions",
    title: "操作",
    width: 88,
    align: "right" as const,
    fixed: "right" as const,
  },
];
</script>

<template>
  <Table
    class="extension-catalog-table"
    :columns="columns"
    :data="packages as ExtensionRow[]"
    empty-text="暂无此类扩展"
    :loading="pending"
    fixed-header
    layout="fixed"
    row-key="repository"
  >
    <template #cell-name="{ row }">
      <span :title="row.name">{{ row.name }}</span>
    </template>
    <template #cell-summary="{ row }">
      <span :title="row.summary">{{ row.summary }}</span>
    </template>
    <template #cell-actions="{ row }">
      <div class="extension-catalog-actions">
        <Button
          square
          size="small"
          variant="ghost"
          icon="ph:eye"
          aria-label="查看详情"
          title="查看详情"
          :disabled="pending"
          @click.stop="emit('detail', row)"
        />
        <Button
          square
          size="small"
          icon="ph:download-simple"
          aria-label="安装"
          title="安装"
          :disabled="pending"
          @click.stop="emit('install', row)"
        />
      </div>
    </template>
  </Table>
</template>

<style scoped>
.extension-catalog-table {
  min-height: 0;
  flex: 1;
}

.extension-catalog-table :deep(.relative) {
  overflow-x: auto;
}

.extension-catalog-table :deep(.inline-block) {
  display: block;
  width: 100%;
  max-width: 100%;
}

.extension-catalog-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-xs);
}
</style>
