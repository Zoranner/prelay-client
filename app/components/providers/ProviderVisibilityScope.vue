<script setup lang="ts">
import { AvatarGroup, type AvatarGroupItem } from "@stellar/ui";
import type { IdentityDirectoryEntry, ProviderListItem } from "~/stores/relay";
import { identityAvatarSrc } from "~/utils/identityAvatar";

const props = defineProps<{
  provider: ProviderListItem;
  identities: IdentityDirectoryEntry[];
}>();

const maxScopeAvatars = 3;

const visibleEntries = computed<IdentityDirectoryEntry[]>(() => {
  const owner: IdentityDirectoryEntry = {
    identity_id: props.provider.owner_identity_id,
    display_name: props.provider.owner_display_name,
  };
  if (props.provider.visibility !== "selected") {
    return [owner];
  }
  const selected = props.provider.selected_identity_ids.map((identityId) => {
    return (
      props.identities.find((entry) => entry.identity_id === identityId) ?? {
        identity_id: identityId,
        display_name: identityId,
      }
    );
  });
  return [owner, ...selected];
});

const cell = computed(() => {
  const entries = visibleEntries.value;
  const owner = entries[0];
  const shared = props.provider.visibility === "all";
  const items: AvatarGroupItem[] = shared
    ? [{ icon: "ph:users-three" }]
    : entries.map((entry) => ({
        src: identityAvatarSrc(entry.identity_id),
        alt: entry.display_name,
      }));
  const label = shared
    ? "全部"
    : props.provider.visibility === "private"
      ? "私有"
      : "";
  const names = entries.map((entry) => entry.display_name).join("、");
  const title = shared
    ? "全部身份可见"
    : props.provider.visibility === "private"
      ? `仅 ${owner?.display_name ?? props.provider.owner_identity_id} 可见`
      : `可见：${names}`;
  return { items, label, title };
});
</script>

<template>
  <AvatarGroup
    :items="cell.items"
    :max="maxScopeAvatars"
    size="small"
    shape="circle"
    variant="capsule"
    :label="cell.label"
    :aria-label="cell.title"
    :title="cell.title"
  />
</template>
