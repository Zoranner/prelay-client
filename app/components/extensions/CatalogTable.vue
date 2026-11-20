<script setup lang="ts">
import { Button, Table, TagGroup, useNotification } from "@stellar/ui";
import type {
  ExtensionArtifactKind,
  ExtensionArtifactSummary,
  ExtensionCatalogPackage,
} from "~/types/relay";
import ExtensionInstallationScope from "~/components/extensions/install/Scope.vue";
import { CONTENTS_COLUMN_WIDTH } from "~/utils/ui/tableLayout";

type ExtensionRow = ExtensionCatalogPackage & Record<string, unknown>;

const props = defineProps<{
  packages: ExtensionCatalogPackage[];
  pending?: boolean;
  installing?: Record<string, { label: string; running: boolean }> | null;
}>();
const emit = defineEmits<{
  detail: [item: ExtensionCatalogPackage];
  install: [item: ExtensionCatalogPackage];
}>();
const notifications = useNotification();
const artifactLabelByKind: Record<ExtensionArtifactKind, string> = {
  skill: "Skill",
  mcp: "MCP",
  hook: "Hook",
  rule: "Rule",
};
const artifactOrder: ExtensionArtifactKind[] = ["skill", "mcp", "hook", "rule"];
const showContents = computed(() =>
  props.packages.some((extension) => extension.artifacts.length > 0),
);
// 有更新的包先显示，避免用户在多页目录里寻找可更新项。
const rows = computed(() => {
  const updateFirst = (extension: ExtensionCatalogPackage) =>
    extension.installAction === "update" ? 0 : 1;
  return [...props.packages].sort(
    (left, right) => updateFirst(left) - updateFirst(right),
  );
});

function repositoryUrl(repository: string) {
  return repository;
}

function artifactLabels(artifacts: ExtensionArtifactSummary[]) {
  const present = new Set(artifacts.map((artifact) => artifact.kind));
  return artifactOrder
    .filter((kind) => present.has(kind))
    .map((kind) => artifactLabelByKind[kind]);
}

function installAction(row: ExtensionCatalogPackage) {
  if (row.installAction === "update")
    return { icon: "ph:arrow-circle-down", label: "更新" };
  if (row.installAction === "partial")
    return { icon: "ph:download-simple", label: "补装" };
  if (row.installAction === "installed")
    return { icon: "ph:check", label: "已安装" };
  return { icon: "ph:download-simple", label: "安装" };
}

function installButtonAppearance(row: ExtensionCatalogPackage) {
  return row.installAction === "update"
    ? { semantic: "primary" as const, variant: "solid" as const }
    : {};
}

function installingJob(row: ExtensionCatalogPackage) {
  return props.installing?.[row.name];
}

function actionIcon(row: ExtensionCatalogPackage) {
  const job = installingJob(row);
  if (!job) return installAction(row).icon;
  return job.running ? "ph:circle-notch" : "ph:clock";
}

function actionLabel(row: ExtensionCatalogPackage) {
  return installingJob(row)?.label ?? installAction(row).label;
}

function actionDisabled(row: ExtensionCatalogPackage) {
  return Boolean(
    props.pending || row.installAction === "installed" || installingJob(row),
  );
}

async function copyRepositoryUrl(repository: string) {
  try {
    await navigator.clipboard.writeText(repositoryUrl(repository));
    notifications.success("已复制仓库链接");
  } catch {
    notifications.error("请手动复制。", { title: "无法访问剪贴板" });
  }
}

type ExtensionColumn = {
  key: string;
  title: string;
  width?: number;
  minWidth?: number;
  ellipsis?: boolean;
  align?: "left" | "center" | "right";
  fixed?: "left" | "right";
};

const baseColumns: ExtensionColumn[] = [
  { key: "name", title: "名称", width: 180, ellipsis: true },
  { key: "version", title: "版本", width: 112, ellipsis: true },
  { key: "installation", title: "安装状态", width: 140 },
  { key: "source", title: "仓库地址", minWidth: 240, ellipsis: true },
  {
    key: "actions",
    title: "操作",
    width: 88,
    align: "right" as const,
    fixed: "right" as const,
  },
];
const columns = computed(() => {
  const result = [...baseColumns];
  if (showContents.value) {
    result.splice(2, 0, {
      key: "contents",
      title: "包含内容",
      width: CONTENTS_COLUMN_WIDTH,
    });
  }
  return result;
});
</script>

<template>
  <Table
    class="extension-catalog-table"
    :columns="columns"
    :data="rows as ExtensionRow[]"
    empty-text="暂无此类扩展"
    :loading="pending"
    fixed-header
    layout="fixed"
    row-key="name"
  >
    <template #cell-name="{ row }">
      <span :title="row.name">{{ row.name }}</span>
    </template>
    <template v-if="showContents" #cell-contents="{ row }">
      <TagGroup :items="artifactLabels(row.artifacts)" />
    </template>
    <template #cell-installation="{ row }">
      <ExtensionInstallationScope :package="row" />
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
          v-bind="installButtonAppearance(row)"
          :class="{
            'extension-catalog-action--busy':
              installingJob(row)?.running === true,
          }"
          :icon="actionIcon(row)"
          :aria-label="actionLabel(row)"
          :title="actionLabel(row)"
          :disabled="actionDisabled(row)"
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

.extension-catalog-action--busy :deep(.iconify) {
  animation: extension-catalog-spin 800ms linear infinite;
}

@keyframes extension-catalog-spin {
  to {
    transform: rotate(360deg);
  }
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
