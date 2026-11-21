<script setup lang="ts">
defineOptions({ name: "ProviderList" });

import { Avatar, Button, Table, Tag } from "@stellar/ui";
import type { Provider } from "~/types/relay";
import type { ProviderQuotaResponse } from "~/types/quota";
import ProviderQuotaCell from "~/components/providers/quota/QuotaCell.vue";
import ProviderSharedQuotaCell from "~/components/providers/quota/SharedQuotaCell.vue";
import { identityAvatarSrc } from "~/utils/ui/avatar";
import { providerProtocolOptions } from "~/utils/provider/capabilities";
import { protocolLabel, protocolTagPalette } from "~/utils/provider/templates";
import { isOwnProvider } from "~/utils/provider/ownership";

type ProviderRow = Provider &
  Record<string, unknown> & {
    isOwn: boolean;
    ping: ReturnType<typeof pingStatus>;
    quota: ProviderQuotaResponse | null;
  };

const props = defineProps<{
  loading?: boolean;
  quotaLoading?: boolean;
  providers: Provider[];
  currentIdentityId?: string | null;
  quotaByProvider: Record<string, ProviderQuotaResponse>;
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
  edit: [provider: Provider];
  ping: [provider: Provider];
  remove: [provider: Provider];
  share: [provider: Provider];
}>();

const columns = [
  { key: "name", title: "名称", width: 200, ellipsis: true },
  { key: "protocols", title: "协议", minWidth: 260, ellipsis: true },
  { key: "status", title: "状态", width: 100 },
  { key: "quota", title: "剩余额度", width: 180 },
  { key: "owner", title: "创建人", width: 128, ellipsis: true },
  {
    key: "actions",
    title: "操作",
    width: 140,
    align: "right" as const,
    fixed: "right" as const,
  },
];
function pingStatus(providerId: string) {
  const state = props.pingStates[providerId];
  if (state?.checking) return { label: "检查中", semantic: "info" as const };
  if (state?.ok) {
    return {
      label: state.latencyMs == null ? "已连接" : `${state.latencyMs} ms`,
      semantic: "success" as const,
    };
  }
  if (state?.ok === false)
    return { label: "连接失败", semantic: "error" as const };
  return { label: "未检查", semantic: undefined };
}

const rows = computed<ProviderRow[]>(() =>
  props.providers.map((provider) => ({
    ...provider,
    isOwn: isOwnProvider(provider, props.currentIdentityId),
    ping: pingStatus(provider.id),
    quota: props.quotaByProvider[provider.id] ?? null,
  })),
);
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
    layout="fixed"
  >
    <template #cell-name="{ row }">
      <div class="provider-name">
        <span>{{ row.name || row.provider_type }}</span>
        <small :title="row.base_url">{{ row.base_url }}</small>
      </div>
    </template>
    <template #cell-protocols="{ row }">
      <div class="protocols">
        <Tag
          v-for="protocol in providerProtocolOptions(row)"
          :key="protocol"
          size="small"
          :palette="protocolTagPalette(protocol)"
        >
          {{ protocolLabel(protocol) }}
        </Tag>
      </div>
    </template>
    <template #cell-status="{ row }">
      <Tag :semantic="row.ping.semantic" size="small">
        {{ row.ping.label }}
      </Tag>
    </template>
    <template #cell-quota="{ row }">
      <ProviderSharedQuotaCell
        v-if="row.quota?.mode === 'shared_limited'"
        :quota="row.quota"
        :loading="quotaLoading"
        :provider-name="row.name || row.provider_type"
      />
      <ProviderQuotaCell
        v-else
        :quota="row.quota"
        :loading="quotaLoading"
        :provider-name="row.name || row.provider_type"
      />
    </template>
    <template #cell-owner="{ row }">
      <div class="owner-entry">
        <Avatar
          class="owner-avatar"
          :src="identityAvatarSrc(row.owner_identity_id)"
          :alt="row.owner_display_name"
          size="small"
          shape="circle"
        />
        <span class="cell-ellipsis" :title="row.owner_display_name">
          {{ row.owner_display_name }}
        </span>
      </div>
    </template>
    <template #cell-actions="{ row }">
      <div class="actions">
        <Button
          square
          size="small"
          icon="ph:heartbeat"
          :disabled="row.ping.label === '检查中'"
          :aria-label="`测试 ${row.name || row.provider_type}`"
          title="测试连接"
          @click.stop="emit('ping', row)"
        />
        <Button
          square
          size="small"
          icon="ph:share-network"
          :disabled="!row.isOwn"
          aria-label="共享供应商"
          title="共享供应商"
          @click.stop="emit('share', row)"
        />
        <Button
          square
          size="small"
          icon="ph:pencil-simple"
          :disabled="!row.isOwn"
          aria-label="编辑供应商"
          title="编辑供应商"
          @click.stop="emit('edit', row)"
        />
        <Button
          square
          size="small"
          semantic="error"
          variant="solid"
          icon="ph:trash"
          :disabled="!row.isOwn"
          aria-label="删除供应商"
          title="删除供应商"
          @click.stop="emit('remove', row)"
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

.cell-ellipsis {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.owner-entry {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-xs);
}

.owner-avatar,
.scope-avatar {
  flex-shrink: 0;
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
