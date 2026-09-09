<script setup lang="ts">
import { Badge, Button, Table, Tag } from "@stellar/ui";
import type { ProviderListItem } from "~/stores/relay";
import { providerProtocolOptions } from "~/utils/providerCapabilities";
import { protocolLabel, protocolTagVariant } from "~/utils/providerTemplates";
import { modelCatalogProviderModels } from "~/utils/modelCatalog";

type ProviderRow = ProviderListItem & Record<string, unknown>;

const props = defineProps<{
  loading?: boolean;
  providers: ProviderListItem[];
  pingStates: Record<
    string,
    {
      checking: boolean;
      ok?: boolean;
      latencyMs?: number | null;
    }
  >;
}>();
const emit = defineEmits<{
  edit: [provider: ProviderListItem];
  ping: [provider: ProviderListItem];
  remove: [provider: ProviderListItem];
  share: [provider: ProviderListItem];
}>();

const columns = [
  { key: "name", title: "名称", width: 220, ellipsis: true },
  { key: "owner", title: "创建人", width: 132, ellipsis: true },
  { key: "protocols", title: "协议", width: 260, ellipsis: true },
  { key: "models", title: "模型", width: 104, align: "right" as const },
  { key: "visibility", title: "共享状态", width: 112 },
  { key: "usage", title: "使用统计", width: 196, ellipsis: true },
  { key: "status", title: "状态", width: 128 },
  {
    key: "actions",
    title: "操作",
    width: 168,
    align: "right" as const,
    fixed: "right" as const,
  },
];
const rows = computed<ProviderRow[]>(() => props.providers as ProviderRow[]);

function pingStatus(providerId: string) {
  const state = props.pingStates[providerId];
  if (state?.checking) return { label: "检查中", variant: "info" as const };
  if (state?.ok) {
    return {
      label: state.latencyMs == null ? "已连接" : `${state.latencyMs} ms`,
      variant: "success" as const,
    };
  }
  if (state?.ok === false)
    return { label: "连接失败", variant: "danger" as const };
  return { label: "未检查", variant: "default" as const };
}

function visibilityLabel(visibility: ProviderListItem["visibility"]) {
  return {
    private: "私有",
    selected: "指定身份",
    all: "全部身份",
  }[visibility];
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
        <span>{{ row.name || row.provider_type }}</span>
        <small :title="row.base_url">{{ row.base_url }}</small>
      </div>
    </template>
    <template #cell-owner="{ row }">
      <span class="cell-ellipsis" :title="row.owner_display_name">
        {{ row.owner_display_name }}
      </span>
    </template>
    <template #cell-protocols="{ row }">
      <div class="protocols">
        <Tag
          v-for="protocol in providerProtocolOptions(row)"
          :key="protocol"
          size="small"
          :variant="protocolTagVariant(protocol)"
        >
          {{ protocolLabel(protocol) }}
        </Tag>
      </div>
    </template>
    <template #cell-models="{ row }">
      {{ modelCatalogProviderModels(row.provider_type).length || "待新增" }}
    </template>
    <template #cell-visibility="{ row }">
      <Tag size="small" :variant="row.can_manage ? 'primary' : 'default'">
        {{ visibilityLabel(row.visibility) }}
      </Tag>
    </template>
    <template #cell-usage="{ row }">
      <span class="usage-summary">
        {{ row.usage?.total_requests ?? 0 }} 次 ·
        {{ row.usage?.input_tokens ?? 0 }} /
        {{ row.usage?.output_tokens ?? 0 }} tokens
      </span>
    </template>
    <template #cell-status="{ row }">
      <Badge :variant="pingStatus(row.id).variant">{{
        pingStatus(row.id).label
      }}</Badge>
    </template>
    <template #cell-actions="{ row }">
      <div class="actions">
        <Button
          square
          size="small"
          icon="ph:heartbeat"
          :disabled="!row.can_manage || pingStatus(row.id).label === '检查中'"
          :aria-label="`测试 ${row.name || row.provider_type}`"
          title="测试连接"
          @click.stop="emit('ping', row)"
        />
        <Button
          square
          size="small"
          icon="ph:pencil-simple"
          :disabled="!row.can_manage"
          aria-label="编辑供应商"
          title="编辑供应商"
          @click.stop="emit('edit', row)"
        />
        <Button
          square
          size="small"
          variant="danger"
          icon="ph:trash"
          :disabled="!row.can_manage"
          aria-label="删除供应商"
          title="删除供应商"
          @click.stop="emit('remove', row)"
        />
        <Button
          square
          size="small"
          icon="ph:share-network"
          :aria-label="`管理 ${row.name || row.provider_type} 共享`"
          title="共享与统计"
          @click.stop="emit('share', row)"
        />
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

.cell-ellipsis,
.usage-summary {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.usage-summary {
  color: var(--st-text-secondary);
  font-family: var(--font-family-mono);
  font-size: 12px;
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
