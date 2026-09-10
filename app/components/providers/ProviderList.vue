<script setup lang="ts">
import { Avatar, Badge, Button, Table, Tag } from "@stellar/ui";
import type { IdentityDirectoryEntry, ProviderListItem } from "~/stores/relay";
import ProviderVisibilityScope from "~/components/providers/ProviderVisibilityScope.vue";
import { identityAvatarSrc } from "~/utils/identityAvatar";
import { providerProtocolOptions } from "~/utils/providerCapabilities";
import { protocolLabel, protocolTagPalette } from "~/utils/providerTemplates";

type ProviderRow = ProviderListItem &
  Record<string, unknown> & { ping: ReturnType<typeof pingStatus> };

const props = defineProps<{
  loading?: boolean;
  providers: ProviderListItem[];
  identities: IdentityDirectoryEntry[];
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
  { key: "protocols", title: "协议", minWidth: 260, ellipsis: true },
  { key: "status", title: "状态", width: 116 },
  { key: "owner", title: "创建人", width: 128, ellipsis: true },
  { key: "visibility", title: "可见范围", width: 140 },
  {
    key: "actions",
    title: "操作",
    width: 108,
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
    ping: pingStatus(provider.id),
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
    <template #cell-visibility="{ row }">
      <ProviderVisibilityScope :provider="row" :identities="identities" />
    </template>
    <template #cell-status="{ row }">
      <Badge :semantic="row.ping.semantic" variant="soft">
        {{ row.ping.label }}
      </Badge>
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
          icon="ph:pencil-simple"
          :disabled="!row.can_manage"
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
          :disabled="!row.can_manage"
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
