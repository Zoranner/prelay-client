<script setup lang="ts">
defineOptions({ name: "AgentItemsList" });

import { Button, Table, Tag, TagGroup, useNotification } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import { CONTENTS_COLUMN_WIDTH } from "~/utils/ui/tableLayout";

type AgentItemRow = AgentItem & { id: string } & Record<string, unknown>;

const props = defineProps<{
  items: AgentItem[];
  pending?: boolean;
  showVersion: boolean;
}>();
const emit = defineEmits<{
  uninstall: [item: AgentItem];
}>();
const notifications = useNotification();

const showMembers = computed(() =>
  props.items.some((item) => item.members.length > 0),
);

const baseColumns = computed(() => [
  { key: "name", title: "名称", width: 180, ellipsis: true },
  { key: "version", title: "版本", width: 120, ellipsis: true },
  { key: "source", title: "来源", width: 96 },
  ...(showMembers.value
    ? [{ key: "members", title: "包含内容", width: CONTENTS_COLUMN_WIDTH }]
    : []),
  { key: "sourcePath", title: "位置", minWidth: 360, ellipsis: true },
  {
    key: "actions",
    title: "操作",
    width: 64,
    align: "right" as const,
    fixed: "right" as const,
  },
]);

const columns = computed(() =>
  baseColumns.value.filter(
    (column) => props.showVersion || column.key !== "version",
  ),
);

const rows = computed<AgentItemRow[]>(() =>
  props.items.map((item) => ({
    ...item,
    id: `${item.kind}-${item.name}-${item.sourcePath}`,
  })),
);
function sourceLabel(source: AgentItem["source"]) {
  return { personal: "个人", team: "团队", system: "内置" }[source];
}

function sourceSemantic(source: AgentItem["source"]) {
  return source === "team"
    ? "primary"
    : source === "system"
      ? "info"
      : undefined;
}

function isBuiltInSource(source: AgentItem["source"]) {
  return source === "system";
}

async function copySourcePath(sourcePath: string) {
  try {
    await navigator.clipboard.writeText(sourcePath);
    notifications.success("已复制来源路径");
  } catch {
    notifications.error("请手动复制。", { title: "无法访问剪贴板" });
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
    <template v-if="showVersion" #cell-version="{ row }">
      {{ row.version || "-" }}
    </template>
    <template v-if="showMembers" #cell-members="{ row }">
      <TagGroup :items="row.members" />
    </template>
    <template #cell-source="{ row }">
      <Tag :semantic="sourceSemantic(row.source)" size="small">
        {{ sourceLabel(row.source) }}
      </Tag>
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
    <template #cell-actions="{ row }">
      <Button
        square
        size="small"
        semantic="error"
        variant="solid"
        icon="ph:trash"
        :aria-label="isBuiltInSource(row.source) ? '内置技能不可卸载' : '卸载'"
        :title="isBuiltInSource(row.source) ? '内置技能不可卸载' : '卸载'"
        :disabled="pending || isBuiltInSource(row.source)"
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
