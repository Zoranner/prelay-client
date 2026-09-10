<script setup lang="ts">
import { Avatar as DiceBearAvatar, Style } from "@dicebear/core";
import cutouts from "@dicebear/styles/cutouts.json";
import { Avatar, Badge, Button, Table, Tag } from "@stellar/ui";
import type { IdentityDirectoryEntry, ProviderListItem } from "~/stores/relay";
import { providerProtocolOptions } from "~/utils/providerCapabilities";
import { protocolLabel, protocolTagPalette } from "~/utils/providerTemplates";

type ProviderRow = ProviderListItem & Record<string, unknown>;

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
const cutoutsStyle = new Style(cutouts);
const maxScopeAvatars = 3;
const emit = defineEmits<{
  edit: [provider: ProviderListItem];
  ping: [provider: ProviderListItem];
  remove: [provider: ProviderListItem];
  share: [provider: ProviderListItem];
}>();

const columns = [
  { key: "name", title: "名称", width: 220, ellipsis: true },
  { key: "protocols", title: "协议", width: 260, ellipsis: true },
  { key: "owner", title: "创建人", width: 168, ellipsis: true },
  { key: "visibility", title: "可见范围", width: 156 },
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

function avatarSrc(identityId: string) {
  return new DiceBearAvatar(cutoutsStyle, {
    seed: identityId,
  }).toDataUri();
}

function ownerEntry(row: ProviderListItem): IdentityDirectoryEntry {
  return {
    identity_id: row.owner_identity_id,
    display_name: row.owner_display_name,
  };
}

function visibleEntries(row: ProviderListItem): IdentityDirectoryEntry[] {
  const owner = ownerEntry(row);
  if (row.visibility !== "selected") return [owner];
  const selected = row.selected_identity_ids.map((identityId) => {
    return (
      props.identities.find((entry) => entry.identity_id === identityId) ?? {
        identity_id: identityId,
        display_name: identityId,
      }
    );
  });
  return [owner, ...selected];
}

function scopeStack(row: ProviderListItem) {
  if (row.visibility === "all") {
    return { entries: [ownerEntry(row)], overflow: 0 };
  }
  const entries = visibleEntries(row);
  return {
    entries: entries.slice(0, maxScopeAvatars),
    overflow: Math.max(0, entries.length - maxScopeAvatars),
  };
}

function scopeTitle(row: ProviderListItem) {
  if (row.visibility === "all") return "全部身份可见";
  const names = visibleEntries(row).map((entry) => entry.display_name);
  return row.visibility === "private"
    ? `仅 ${names[0] ?? row.owner_identity_id} 可见`
    : `可见：${names.join("、")}`;
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
          :src="avatarSrc(row.owner_identity_id)"
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
      <div class="visibility-scope" :title="scopeTitle(row)">
        <Avatar
          v-for="entry in scopeStack(row).entries"
          :key="entry.identity_id"
          class="scope-avatar"
          :src="avatarSrc(entry.identity_id)"
          :alt="entry.display_name"
          size="small"
          shape="circle"
        />
        <span v-if="scopeStack(row).overflow > 0" class="scope-overflow">
          +{{ scopeStack(row).overflow }}
        </span>
        <span v-else-if="row.visibility === 'all'" class="scope-all">全部</span>
      </div>
    </template>
    <template #cell-status="{ row }">
      <Badge :semantic="pingStatus(row.id).semantic" variant="soft">{{
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
          semantic="error"
          variant="solid"
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

.visibility-scope {
  display: flex;
  min-width: 0;
  align-items: center;
}

.visibility-scope > :not(:first-child) {
  margin-left: -8px;
}

.scope-avatar,
.scope-overflow,
.scope-all {
  box-shadow: 0 0 0 2px var(--st-bg-surface);
}

.scope-overflow,
.scope-all {
  display: inline-flex;
  height: 24px;
  padding: 0 7px;
  flex-shrink: 0;
  align-items: center;
  color: var(--st-text-secondary);
  background: var(--st-bg-elevated);
  border: 1px solid var(--st-border-divider);
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
}

.scope-overflow {
  font-family: var(--font-family-mono);
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
