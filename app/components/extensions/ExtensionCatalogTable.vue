<script setup lang="ts">
import { Button, Table, useNotification } from "@stellar/ui";
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
const notifications = useNotification();

function repositoryUrl(repository: string) {
  return repository;
}

async function copyRepositoryUrl(repository: string) {
  try {
    await navigator.clipboard.writeText(repositoryUrl(repository));
    notifications.success("已复制仓库链接");
  } catch {
    notifications.danger("请手动复制。", { title: "无法访问剪贴板" });
  }
}

const columns = [
  { key: "name", title: "名称", width: 180, ellipsis: true },
  { key: "version", title: "版本", width: 112, ellipsis: true },
  { key: "source", title: "参考来源", minWidth: 240, ellipsis: true },
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
    row-key="name"
  >
    <template #cell-name="{ row }">
      <span :title="row.name">{{ row.name }}</span>
    </template>
    <template #cell-source="{ row }">
      <div class="extension-catalog-source">
        <a
          :href="repositoryUrl(row.repository)"
          target="_blank"
          rel="noopener noreferrer"
          class="extension-catalog-link"
          :title="repositoryUrl(row.repository)"
          @click.stop
        >
          {{ repositoryUrl(row.repository).replace("https://", "") }}
        </a>
        <Button
          square
          size="small"
          variant="ghost"
          icon="ph:copy"
          aria-label="复制仓库链接"
          title="复制仓库链接"
          @click.stop="copyRepositoryUrl(row.repository)"
        />
      </div>
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

.extension-catalog-link {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: var(--color-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.extension-catalog-link:hover {
  color: var(--color-primary);
}

.extension-catalog-source {
  display: flex;
  width: 100%;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-xs);
}
</style>
