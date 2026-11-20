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
  ShareQuotaWindow,
  ShareQuotaWindowKind,
} from "~/types/quota";
import type { EditableShare, EditableWindow, ShareMode } from "./types";
import RuleEditor from "./RuleEditor.vue";
import UserTransfer from "./UserTransfer.vue";
import { ruleFromWindows, windowsFromRule } from "./rule";
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
const quotaRule = ref<QuotaRule>(ruleFromWindows(emptyWindows()));
const quotaWindows = ref<EditableWindow[]>(emptyWindows());
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
const windowNames: Record<EditableWindow["kind"], string> = {
  day: "日",
  week: "周",
  month: "月",
};

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
      quotaWindows.value = emptyWindows();
      quotaRule.value = ruleFromWindows(quotaWindows.value);
      return;
    }
    visibility.value = sharing.visibility;
    users.value = sharing.users.map(toEditableUser);
    quotaMode.value = sharing.quota?.length ? "limited" : "unlimited";
    quotaWindows.value = windowsFromQuota(sharing.quota);
    quotaRule.value = ruleFromWindows(quotaWindows.value);
    savedSnapshot.value = snapshot();
  },
  { immediate: true },
);

function defaultWindow(kind: EditableWindow["kind"]): EditableWindow {
  return {
    kind,
    enabled: kind === "day",
    limit: "",
    accumulate: false,
    reset_time: "00:00",
    reset_weekday: 1,
    reset_day: 1,
  };
}

function emptyWindows() {
  return [defaultWindow("day"), defaultWindow("week"), defaultWindow("month")];
}

function windowsFromQuota(quota: ShareQuotaWindow[] | null) {
  const windows = emptyWindows();
  for (const rule of quota ?? []) {
    const window = windows.find((candidate) => candidate.kind === rule.window);
    if (!window) continue;
    window.enabled = true;
    window.limit = String(rule.limit);
    window.accumulate = rule.accumulate;
    window.reset_time = rule.reset_time;
    if (rule.reset_weekday != null) window.reset_weekday = rule.reset_weekday;
    if (rule.reset_day != null) window.reset_day = rule.reset_day;
  }
  return windows;
}

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
  quotaWindows.value = windowsFromRule(rule, quotaWindows.value);
}

function updateQuotaMode(value: string | number | boolean) {
  if (typeof value !== "string") return;
  quotaMode.value = value === "limited" ? "limited" : "unlimited";
}

function toQuotaRule(window: EditableWindow): ShareQuotaWindow | null {
  const limit = Number(window.limit);
  if (!Number.isInteger(limit) || limit <= 0) {
    notifications.error(`${windowNames[window.kind]}额度必须是正整数。`);
    return null;
  }
  if (!/^\d{2}:\d{2}$/.test(window.reset_time)) {
    notifications.error(`${windowNames[window.kind]}清零时间必须是 HH:MM。`);
    return null;
  }
  const resetWeekday = Number(window.reset_weekday);
  const resetDay = Number(window.reset_day);
  if (
    window.kind === "week" &&
    (!Number.isInteger(resetWeekday) || resetWeekday < 1 || resetWeekday > 7)
  ) {
    notifications.error("周额度清零星期必须在周一到周日之间。");
    return null;
  }
  if (
    window.kind === "month" &&
    (!Number.isInteger(resetDay) || resetDay < 1 || resetDay > 28)
  ) {
    notifications.error("月额度清零日期必须在 1 到 28 之间。");
    return null;
  }
  return {
    window: window.kind as ShareQuotaWindowKind,
    limit,
    accumulate: window.accumulate,
    reset_time: window.reset_time,
    reset_weekday: window.kind === "week" ? resetWeekday : null,
    reset_day: window.kind === "month" ? resetDay : null,
  };
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
    const quota = quotaWindows.value
      .filter((window) => window.enabled)
      .map(toQuotaRule);
    if (quota.some((rule) => rule === null)) return;
    if (!quota.length) {
      notifications.error("限额共享至少启用一个窗口。");
      return;
    }
    input.quota = quota as ShareQuotaWindow[];
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
