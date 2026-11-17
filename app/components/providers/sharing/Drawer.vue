<script setup lang="ts">
defineOptions({ name: "ProviderSharingDrawer" });

import { Avatar, Button, Drawer, Select, useNotification } from "@stellar/ui";
import type { IdentityDirectoryEntry, ProviderListItem } from "~/types/relay";
import type {
  ProviderShare,
  ProviderSharing,
  ProviderSharingInput,
  QuotaWindowKind,
  ShareQuotaWindow,
} from "~/types/quota";
import { identityAvatarSrc } from "~/utils/ui/avatar";
import type { EditableShare, EditableWindow } from "./types";
import WindowEditor from "./WindowEditor.vue";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  provider: ProviderListItem | null;
  sharing: ProviderSharing | null;
  identities: IdentityDirectoryEntry[];
  pending?: boolean;
  loading?: boolean;
}>();
const emit = defineEmits<{
  "save-sharing": [input: ProviderSharingInput];
  close: [];
}>();

const notifications = useNotification();
const shares = ref<EditableShare[]>([]);
const selectedIdentityId = ref("");

const modeOptions = [
  { label: "不限额", value: "unlimited" },
  { label: "限额", value: "limited" },
];
const windowNames: Record<EditableWindow["kind"], string> = {
  day: "日",
  week: "周",
  month: "月",
};

const availableIdentities = computed(() => {
  const ownerId = props.provider?.owner_identity_id;
  const used = new Set(shares.value.map((share) => share.identity_id));
  return props.identities.filter(
    (identity) =>
      identity.identity_id !== ownerId && !used.has(identity.identity_id),
  );
});

const identityOptions = computed(() =>
  availableIdentities.value.map((identity) => ({
    label: identity.display_name,
    value: identity.identity_id,
  })),
);

watch(
  () => [props.sharing, visible.value] as const,
  ([sharing, isVisible]) => {
    if (!isVisible) {
      shares.value = [];
      return;
    }
    if (!sharing) {
      shares.value = [];
      return;
    }
    shares.value = sharing.shares.map(toEditableShare);
    selectedIdentityId.value = "";
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

function toEditableShare(share: ProviderShare): EditableShare {
  const windows = emptyWindows();
  if (share.quota?.length) {
    for (const rule of share.quota) {
      const window = windows.find(
        (candidate) => candidate.kind === rule.window,
      );
      if (!window) continue;
      window.enabled = true;
      window.limit = String(rule.limit);
      window.accumulate = rule.accumulate;
      window.reset_time = rule.reset_time;
      if (rule.reset_weekday != null) window.reset_weekday = rule.reset_weekday;
      if (rule.reset_day != null) window.reset_day = rule.reset_day;
    }
  }
  return {
    identity_id: share.identity_id,
    display_name: share.display_name,
    mode: share.quota?.length ? "limited" : "unlimited",
    windows,
  };
}

function addShare() {
  const identity = props.identities.find(
    (candidate) => candidate.identity_id === selectedIdentityId.value,
  );
  if (!identity) return;
  shares.value.push({
    identity_id: identity.identity_id,
    display_name: identity.display_name,
    mode: "unlimited",
    windows: emptyWindows(),
  });
  selectedIdentityId.value = "";
}

function removeShare(index: number) {
  shares.value.splice(index, 1);
}

function updateMode(share: EditableShare, value: string | number | boolean) {
  if (typeof value !== "string") return;
  share.mode = value === "limited" ? "limited" : "unlimited";
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
    window: window.kind as QuotaWindowKind,
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
    shares: [],
  };
  for (const share of shares.value) {
    if (share.mode === "unlimited") {
      input.shares.push({ identity_id: share.identity_id, quota: null });
      continue;
    }
    const quota = share.windows
      .filter((window) => window.enabled)
      .map(toQuotaRule);
    if (quota.some((rule) => rule === null)) return;
    if (!quota.length) {
      notifications.error("限额共享至少启用一个窗口。");
      return;
    }
    input.shares.push({
      identity_id: share.identity_id,
      quota: quota as ShareQuotaWindow[],
    });
  }
  emit("save-sharing", input);
}

function close() {
  visible.value = false;
  emit("close");
}
</script>

<template>
  <Drawer
    :visible="visible"
    :title="provider ? `${provider.name} · 共享设置` : '共享设置'"
    size="xlarge"
    :blocked="pending || loading"
    @update:visible="
      (nextVisible) => (nextVisible ? (visible = true) : close())
    "
  >
    <div class="sharing-drawer">
      <section class="drawer-section">
        <div class="section-heading">
          <div>
            <h3>共享给</h3>
            <p>
              创建人：{{ provider?.owner_display_name || "暂无" }}
              <span v-if="!provider?.can_manage" class="readonly-tag"
                >只读</span
              >
            </p>
          </div>
        </div>
        <div v-if="provider?.can_manage && !loading" class="add-share">
          <Select
            v-model="selectedIdentityId"
            :options="identityOptions"
            placeholder="选择身份"
            :disabled="pending || !identityOptions.length"
          />
          <Button
            :disabled="
              pending || !selectedIdentityId || !identityOptions.length
            "
            icon="ph:plus"
            @click="addShare"
          >
            添加
          </Button>
        </div>
        <p
          v-if="provider?.can_manage && !loading && !identityOptions.length"
          class="empty-text"
        >
          暂无可共享的其他身份。
        </p>
      </section>

      <section
        v-for="(share, index) in shares"
        :key="share.identity_id"
        class="drawer-section share-card"
      >
        <div class="share-header">
          <div class="share-identity">
            <Avatar
              :src="identityAvatarSrc(share.identity_id)"
              :alt="share.display_name"
              size="small"
              shape="circle"
            />
            <strong>{{ share.display_name }}</strong>
          </div>
          <div class="share-actions">
            <Select
              class="share-mode"
              :model-value="share.mode"
              :options="modeOptions"
              :disabled="pending || !provider?.can_manage"
              @update:model-value="updateMode(share, $event)"
            />
            <Button
              v-if="provider?.can_manage"
              square
              size="small"
              semantic="error"
              variant="ghost"
              icon="ph:trash"
              aria-label="移除共享"
              title="移除共享"
              :disabled="pending"
              @click="removeShare(index)"
            />
          </div>
        </div>

        <WindowEditor
          v-if="share.mode === 'limited'"
          :windows="share.windows"
          :disabled="pending || !provider?.can_manage"
        />
      </section>

      <p v-if="loading" class="empty-text">正在加载共享设置...</p>
      <p v-else-if="!shares.length" class="empty-text">
        还没有共享给任何身份。
      </p>
    </div>
    <template #footer>
      <Button @click="close">关闭</Button>
      <Button
        v-if="provider?.can_manage"
        semantic="primary"
        variant="solid"
        :disabled="pending || loading || !sharing"
        @click="saveSharing"
      >
        {{ pending ? "保存中..." : "保存共享设置" }}
      </Button>
    </template>
  </Drawer>
</template>

<style scoped>
.sharing-drawer,
.drawer-section {
  display: grid;
  gap: var(--spacing-md);
}

.sharing-drawer {
  min-height: 100%;
  padding: var(--spacing-lg);
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.section-heading h3,
.section-heading p {
  margin: 0;
}

.section-heading h3 {
  color: var(--st-text-primary);
  font-size: 15px;
}

.section-heading p,
.empty-text,
.readonly-tag {
  color: var(--st-text-secondary);
}

.section-heading p {
  margin-top: var(--spacing-xs);
}

.readonly-tag {
  margin-left: var(--spacing-xs);
  font-size: 11px;
}

.add-share {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--spacing-sm);
  align-items: end;
}

.share-card {
  padding: var(--spacing-md);
  border: 1px solid var(--st-border-divider);
  border-radius: var(--st-radius-md);
  background: var(--st-bg-elevated);
}

.share-header,
.share-actions,
.share-identity {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.share-header {
  justify-content: space-between;
}

.share-mode {
  width: 112px;
  flex: 0 0 auto;
}

.share-identity {
  min-width: 0;
}

.share-identity strong {
  overflow: hidden;
  color: var(--st-text-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
