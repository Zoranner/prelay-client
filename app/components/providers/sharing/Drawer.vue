<script setup lang="ts">
defineOptions({ name: "ProviderSharingDrawer" });

import {
  Button,
  Drawer,
  Select,
  useConfirm,
  useNotification,
} from "@stellar/ui";
import type {
  IdentityDirectoryEntry,
  Provider,
  ProviderVisibility,
} from "~/types/relay";
import type {
  ProviderShareUser,
  ProviderSharing,
  ProviderSharingInput,
} from "~/types/quota";
import type { EditableShare, ShareMode } from "./types";
import RuleEditor from "./RuleEditor.vue";
import UserTransfer from "./UserTransfer.vue";
import { quotaFromRule, ruleFromQuota } from "./rule";
import type { QuotaRule } from "./rule";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  provider: Provider | null;
  sharing: ProviderSharing | null;
  identities: IdentityDirectoryEntry[];
  pending?: boolean;
  loading?: boolean;
  failed?: boolean;
}>();
const emit = defineEmits<{
  "save-sharing": [input: ProviderSharingInput];
  close: [];
}>();

const notifications = useNotification();
const { confirm: confirmAction } = useConfirm();
const users = ref<EditableShare[]>([]);
const visibility = ref<ProviderVisibility>("private");
const quotaMode = ref<ShareMode>("unlimited");
const quotaRule = ref<QuotaRule>(ruleFromQuota(null));
const savedSnapshot = ref("");

const visibilityOptions = [
  { label: "不共享", value: "private" },
  { label: "指定用户", value: "selected" },
  { label: "所有用户", value: "all" },
];

const modeOptions = [
  { label: "不限额", value: "unlimited" },
  { label: "限额", value: "limited" },
];
function compareUsers(left: EditableShare, right: EditableShare) {
  return (
    left.display_name.localeCompare(right.display_name, "zh-CN", {
      numeric: true,
      sensitivity: "base",
    }) || left.identity_id.localeCompare(right.identity_id)
  );
}

const selectableUsers = computed(() => {
  const ownerId = props.provider?.owner_identity_id;
  const byId = new Map<string, EditableShare>();
  for (const identity of props.identities) {
    if (identity.identity_id === ownerId) continue;
    byId.set(identity.identity_id, {
      identity_id: identity.identity_id,
      display_name: identity.display_name,
    });
  }
  for (const user of users.value) {
    if (!byId.has(user.identity_id)) byId.set(user.identity_id, user);
  }
  const selected = users.value
    .map((user) => byId.get(user.identity_id) ?? user)
    .sort(compareUsers);
  const selectedIds = new Set(selected.map((user) => user.identity_id));
  const remaining = [...byId.values()]
    .filter((user) => !selectedIds.has(user.identity_id))
    .sort(compareUsers);
  return [...selected, ...remaining];
});

watch(
  () => [props.sharing, visible.value] as const,
  ([sharing, isVisible]) => {
    if (!isVisible) {
      users.value = [];
      return;
    }
    if (!sharing) {
      users.value = [];
      visibility.value = "private";
      quotaMode.value = "unlimited";
      quotaRule.value = ruleFromQuota(null);
      return;
    }
    visibility.value = sharing.visibility;
    users.value = sharing.users.map(toEditableUser);
    quotaMode.value = sharing.quota ? "limited" : "unlimited";
    quotaRule.value = ruleFromQuota(sharing.quota);
    savedSnapshot.value = snapshot();
  },
  { immediate: true },
);

function toEditableUser(user: ProviderShareUser): EditableShare {
  return { identity_id: user.identity_id, display_name: user.display_name };
}

function snapshot() {
  return JSON.stringify({
    visibility: visibility.value,
    users: users.value.map((user) => user.identity_id),
    quotaMode: quotaMode.value,
    quota: quotaRule.value,
  });
}

const dirty = computed(
  () => Boolean(props.sharing) && snapshot() !== savedSnapshot.value,
);
const saveDisabled = computed(
  () =>
    props.pending ||
    props.loading ||
    props.failed ||
    !props.sharing ||
    (visibility.value === "selected" && !users.value.length),
);

function setVisibility(value: string | number | boolean) {
  if (props.pending || typeof value !== "string") return;
  visibility.value = value as ProviderVisibility;
}

function addUsers(identityIds: string[]) {
  const ids = new Set(identityIds);
  users.value.push(
    ...selectableUsers.value
      .filter((identity) => ids.has(identity.identity_id))
      .filter(
        (identity) =>
          !users.value.some(
            (user) => user.identity_id === identity.identity_id,
          ),
      ),
  );
}

function removeUsers(identityIds: string[]) {
  const ids = new Set(identityIds);
  users.value = users.value.filter((user) => !ids.has(user.identity_id));
}

function applyRule(rule: QuotaRule) {
  quotaRule.value = rule;
}

function updateQuotaMode(value: string | number | boolean) {
  if (typeof value !== "string") return;
  quotaMode.value = value === "limited" ? "limited" : "unlimited";
}

function saveSharing() {
  if (!props.sharing) {
    notifications.error("共享配置尚未加载。");
    return;
  }
  const input: ProviderSharingInput = {
    visibility: visibility.value,
    users:
      visibility.value === "selected"
        ? users.value.map((user) => ({ identity_id: user.identity_id }))
        : [],
    quota: null,
  };
  if (visibility.value !== "private" && quotaMode.value === "limited") {
    const quota = quotaFromRule(quotaRule.value);
    if (!quota) {
      notifications.error("每天的共享额度必须大于 0。");
      return;
    }
    input.quota = quota;
  }
  emit("save-sharing", input);
}

async function close() {
  if (dirty.value) {
    const confirmed = await confirmAction({
      title: "放弃共享设置",
      message: "关闭后本次修改不会保存。",
      confirmText: "放弃修改",
      danger: true,
    });
    if (!confirmed) return;
  }
  visible.value = false;
  emit("close");
}
</script>

<template>
  <Drawer
    :visible="visible"
    title="共享设置"
    size="xlarge"
    :blocked="pending || loading"
    @update:visible="
      (nextVisible) => (nextVisible ? (visible = true) : close())
    "
  >
    <div class="sharing-drawer">
      <div class="sharing-field">
        <span class="sharing-field__label">共享范围</span>
        <Select
          :model-value="visibility"
          :options="visibilityOptions"
          :disabled="pending"
          @update:model-value="setVisibility"
        />
      </div>
      <section v-if="visibility !== 'private'" class="sharing-users">
        <UserTransfer
          :users="selectableUsers"
          :selected="visibility === 'all' ? selectableUsers : users"
          :disabled="pending"
          :readonly="visibility === 'all'"
          @add="addUsers"
          @remove="removeUsers"
        />
      </section>
      <div v-if="visibility !== 'private'" class="sharing-field">
        <span class="sharing-field__label">共享额度</span>
        <Select
          :model-value="quotaMode"
          :options="modeOptions"
          :disabled="pending"
          @update:model-value="updateQuotaMode"
        />
      </div>
      <RuleEditor
        v-if="visibility !== 'private' && quotaMode === 'limited'"
        :model-value="quotaRule"
        :disabled="pending"
        @update:model-value="applyRule"
      />
    </div>
    <template #footer>
      <Button :disabled="pending" @click="close">关闭</Button>
      <Button
        semantic="primary"
        variant="solid"
        :disabled="saveDisabled"
        @click="saveSharing"
      >
        {{ pending ? "保存中..." : "保存" }}
      </Button>
    </template>
  </Drawer>
</template>

<style scoped src="./Drawer.css"></style>
