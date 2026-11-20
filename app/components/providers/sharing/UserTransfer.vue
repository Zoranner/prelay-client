<script setup lang="ts">
import { Avatar, Button, EmptyState } from "@stellar/ui";
import { identityAvatarSrc } from "~/utils/ui/avatar";
import type { EditableShare } from "./types";

defineOptions({ name: "ProviderSharingUserTransfer" });

const props = defineProps<{
  users: EditableShare[];
  selected: EditableShare[];
  disabled?: boolean;
  readonly?: boolean;
}>();
const emit = defineEmits<{
  add: [identityIds: string[]];
  remove: [identityIds: string[]];
}>();

const selectedIds = computed(
  () => new Set(props.selected.map((user) => user.identity_id)),
);

function toggle(user: EditableShare) {
  if (props.disabled || props.readonly) return;
  if (selectedIds.value.has(user.identity_id)) {
    emit("remove", [user.identity_id]);
    return;
  }
  emit("add", [user.identity_id]);
}
</script>

<template>
  <div
    v-if="users.length"
    class="user-picker"
    :class="{ 'user-picker--readonly': readonly }"
  >
    <Button
      v-for="user in users"
      :key="user.identity_id"
      class="user-chip"
      :class="{ 'user-chip--selected': selectedIds.has(user.identity_id) }"
      size="default"
      variant="ghost"
      :active="selectedIds.has(user.identity_id)"
      :disabled="disabled"
      :aria-disabled="readonly || undefined"
      :tabindex="readonly ? -1 : undefined"
      :aria-pressed="selectedIds.has(user.identity_id)"
      @click="toggle(user)"
    >
      <span class="user-chip__body">
        <Avatar
          :src="identityAvatarSrc(user.identity_id)"
          :alt="user.display_name"
          size="small"
          shape="circle"
        />
        <span class="user-chip__name">{{ user.display_name }}</span>
      </span>
    </Button>
  </div>
  <EmptyState v-else size="small" title="没有可共享的其他用户" />
</template>

<style scoped>
.user-picker {
  display: flex;
  max-height: 260px;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
  align-content: flex-start;
  padding: var(--spacing-xs);
  overflow-y: auto;
}

.user-picker--readonly .user-chip {
  pointer-events: none;
  cursor: default;
}

.user-chip {
  border-radius: 999px;
  padding-left: 3px;
  padding-right: var(--spacing-md);
}

.user-chip--selected:hover {
  border-color: var(--st-border-active);
  background: linear-gradient(
    to bottom,
    color-mix(in srgb, var(--st-primary) 10%, var(--st-bg-surface)),
    color-mix(in srgb, var(--st-primary) 5%, var(--st-bg-elevated))
  );
}

.user-chip__body {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-sm);
}

.user-chip__name {
  overflow: hidden;
  max-width: 180px;
  color: var(--st-text-primary);
  font-size: 13px;
  line-height: 1.5;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
