<script setup lang="ts">
import { Button, Table, TagGroup } from "@stellar/ui";
import type { EndpointModel, RelayEndpoint } from "~/stores/relay";
import { modelCatalogLabel } from "~/utils/modelCatalog";

type EndpointRow = RelayEndpoint & Record<string, unknown>;

const props = defineProps<{
  endpoints: RelayEndpoint[];
  pending?: boolean;
}>();
const emit = defineEmits<{
  edit: [item: RelayEndpoint];
  remove: [item: RelayEndpoint];
  regenerate: [item: RelayEndpoint];
  copy: [value: string];
}>();

const columns = [
  { key: "name", title: "名称", width: 280, ellipsis: true },
  { key: "token", title: "API Token", width: 240, ellipsis: true },
  { key: "models", title: "模型" },
  {
    key: "actions",
    title: "操作",
    width: 108,
    align: "right" as const,
    fixed: "right" as const,
  },
];
const rows = computed<EndpointRow[]>(() => props.endpoints as EndpointRow[]);

function maskToken(token: string) {
  if (token.length <= 10) return "*".repeat(token.length);
  return `${token.slice(0, 6)}...${token.slice(-4)}`;
}

function modelLabel(model: EndpointModel) {
  return (
    model.display_name?.trim() ||
    modelCatalogLabel(model.model_name || model.upstream_model)
  );
}

function modelLabels(row: EndpointRow) {
  return row.models.map(modelLabel);
}
</script>

<template>
  <Table
    :columns="columns"
    class="endpoint-table"
    :data="rows"
    empty-text="暂无接入点"
    :loading="pending"
    layout="auto"
    row-key="id"
    fixed-header
  >
    <template #cell-name="{ row }">
      <div class="endpoint-name">
        <span>{{ row.name }}</span>
        <small :title="row.id">{{ row.id }}</small>
      </div>
    </template>
    <template #cell-token="{ row }">
      <Button
        variant="ghost"
        size="small"
        icon="ph:copy"
        icon-position="right"
        @click.stop="emit('copy', row.token)"
      >
        {{ maskToken(row.token) }}
      </Button>
    </template>
    <template #cell-models="{ row }">
      <TagGroup :items="modelLabels(row)" />
    </template>
    <template #cell-actions="{ row }">
      <div class="actions">
        <Button
          square
          size="small"
          icon="ph:key"
          :disabled="pending"
          aria-label="重置 API Token"
          title="重置 API Token"
          @click.stop="emit('regenerate', row)"
        />
        <Button
          square
          size="small"
          icon="ph:pencil-simple"
          :disabled="pending"
          aria-label="编辑接入点"
          title="编辑接入点"
          @click.stop="emit('edit', row)"
        />
        <Button
          square
          size="small"
          semantic="error"
          variant="solid"
          icon="ph:trash"
          :disabled="pending"
          aria-label="删除接入点"
          title="删除接入点"
          @click.stop="emit('remove', row)"
        />
      </div>
    </template>
  </Table>
</template>

<style scoped>
.endpoint-name {
  display: grid;
  min-width: 0;
  gap: 2px;
}

.endpoint-name > span,
.endpoint-name > small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.endpoint-name > span {
  color: var(--st-text-primary);
  font-weight: 600;
}

.endpoint-name > small {
  color: var(--st-text-muted);
  font-family: var(--font-family-mono);
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-xs);
}

.endpoint-table {
  min-height: 0;
  flex: 1;
}
</style>
