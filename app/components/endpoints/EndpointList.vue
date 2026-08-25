<script setup lang="ts">
import { Button, Table, Tag } from "@stellar/ui";
import type { RelayEndpoint } from "~/stores/relay";

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
    width: 132,
    align: "right" as const,
    fixed: "right" as const,
  },
];
const rows = computed<EndpointRow[]>(() => props.endpoints as EndpointRow[]);

function maskToken(token: string) {
  if (token.length <= 10) return "*".repeat(token.length);
  return `${token.slice(0, 6)}...${token.slice(-4)}`;
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
      <div class="endpoint-models">
        <Tag
          v-for="model in row.models.slice(0, 3)"
          :key="model.id ?? `${model.provider_id}-${model.model_name}`"
          size="small"
        >
          {{ model.model_name }}
        </Tag>
        <Tag v-if="row.models.length > 3" size="small"
          >+{{ row.models.length - 3 }}</Tag
        >
      </div>
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
          variant="danger"
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

.endpoint-models {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-xs);
  white-space: nowrap;
}

.endpoint-table {
  min-height: 0;
  flex: 1;
}
</style>
