<script setup lang="ts">
import { Badge, Button, Table, Tag } from "stellar-ui";
import type { Provider } from "~/stores/relay";
import { providerProtocolOptions } from "~/utils/providerCapabilities";
import {
  protocolLabel,
  providerLabel,
  PROVIDER_TEMPLATE_GROUPS,
} from "~/utils/providerTemplates";

type ProviderRow = Provider & Record<string, unknown>;

const props = defineProps<{
  loading?: boolean;
  providers: Provider[];
  pingStates: Record<string, {
    checking: boolean;
    ok?: boolean;
    latencyMs?: number | null;
  }>;
}>();
const emit = defineEmits<{
  edit: [provider: Provider];
  ping: [provider: Provider];
  remove: [provider: Provider];
}>();

const columns = [
  { key: "name", title: "名称", width: 220, ellipsis: true },
  { key: "category", title: "类型", width: 88 },
  { key: "protocols", title: "协议", width: 260, ellipsis: true },
  { key: "models", title: "模型", width: 104, align: "right" as const },
  { key: "status", title: "状态", width: 128 },
  { key: "actions", title: "操作", width: 132, align: "right" as const, fixed: "right" as const },
];
const rows = computed<ProviderRow[]>(() => props.providers as ProviderRow[]);

function providerCategory(providerType: string) {
  const group = PROVIDER_TEMPLATE_GROUPS.find((item) =>
    item.options.some((option) => option.providerType === providerType),
  );
  if (group?.label === "套餐服务") return "套餐";
  if (group?.label === "API 服务") return "API";
  return "自定义";
}

function pingStatus(providerId: string) {
  const state = props.pingStates[providerId];
  if (state?.checking) return { label: "检查中", variant: "info" as const };
  if (state?.ok) {
    return {
      label: state.latencyMs == null ? "已连接" : `${state.latencyMs} ms`,
      variant: "success" as const,
    };
  }
  if (state?.ok === false) return { label: "连接失败", variant: "danger" as const };
  return { label: "未检查", variant: "default" as const };
}
</script>

<template>
  <Table
    :columns="columns"
    class="provider-table"
    :data="rows"
    empty-text="暂无供应商"
    :loading="loading"
    row-key="id"
    fixed-header
  >
    <template #cell-name="{ row }">
      <div class="provider-name">
        <span>{{ row.name || providerLabel(row.provider_type) }}</span>
        <small :title="row.base_url">{{ row.base_url }}</small>
      </div>
    </template>
    <template #cell-category="{ row }">
      <Tag size="small">{{ providerCategory(row.provider_type) }}</Tag>
    </template>
    <template #cell-protocols="{ row }">
      <div class="protocols">
        <Tag
          v-for="protocol in providerProtocolOptions(row)"
          :key="protocol"
          size="small"
          variant="primary"
        >
          {{ protocolLabel(protocol) }}
        </Tag>
      </div>
    </template>
    <template #cell-models="{ row }">
      {{ row.models.length || "待新增" }}
    </template>
    <template #cell-status="{ row }">
      <Badge :variant="pingStatus(row.id).variant">{{ pingStatus(row.id).label }}</Badge>
    </template>
    <template #cell-actions="{ row }">
      <div class="actions">
        <Button
          square
          size="small"
          icon="ph:heartbeat"
          :disabled="pingStatus(row.id).label === '检查中'"
          :aria-label="`测试 ${row.name || providerLabel(row.provider_type)}`"
          title="测试连接"
          @click.stop="emit('ping', row)"
        />
        <Button square size="small" icon="ph:pencil-simple" aria-label="编辑供应商" title="编辑供应商" @click.stop="emit('edit', row)" />
        <Button square size="small" variant="danger" icon="ph:trash" aria-label="删除供应商" title="删除供应商" @click.stop="emit('remove', row)" />
      </div>
    </template>
  </Table>
</template>

<style scoped>
.provider-name {
  display: grid;
  min-width: 0;
  gap: 2px;
}

.provider-name > span,
.provider-name > small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.provider-name > span {
  color: var(--st-text-primary);
  font-weight: 600;
}

.provider-name > small {
  color: var(--st-text-muted);
  font-family: var(--font-family-mono);
}

.protocols,
.actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.protocols {
  min-width: 0;
  overflow: hidden;
}

.actions {
  justify-content: flex-end;
}

.provider-table {
  min-height: 0;
  flex: 1;
}
</style>
