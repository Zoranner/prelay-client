<script setup lang="ts">
import {
  Button,
  Drawer,
  RadioGroup,
  Select,
  StatCard,
  Table,
  Tag,
} from "@stellar/ui";
import type {
  IdentityDirectoryEntry,
  ProviderListItem,
  ProviderSharing,
  ProviderSharingInput,
  ProviderUsage,
} from "~/stores/relay";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  provider: ProviderListItem | null;
  sharing: ProviderSharing | null;
  identities: IdentityDirectoryEntry[];
  usage: ProviderUsage | null;
  pending?: boolean;
  loading?: boolean;
}>();
const emit = defineEmits<{
  "save-sharing": [input: ProviderSharingInput];
  close: [];
}>();

const visibility = ref<ProviderSharing["visibility"]>("private");
const selectedIdentityIds = ref<string[]>([]);
const visibilityOptions = [
  { label: "私有", value: "private" },
  { label: "指定身份", value: "selected" },
  { label: "全部身份", value: "all" },
];
const identityOptions = computed(() =>
  props.identities
    .filter(
      (identity) => identity.identity_id !== props.provider?.owner_identity_id,
    )
    .map((identity) => ({
      label: identity.display_name,
      value: identity.identity_id,
    })),
);
const usageColumns = [
  { key: "display_name", title: "使用者", width: 150, ellipsis: true },
  { key: "request_count", title: "请求", width: 84, align: "right" as const },
  { key: "input_tokens", title: "输入", width: 96, align: "right" as const },
  { key: "output_tokens", title: "输出", width: 96, align: "right" as const },
  { key: "total_tokens", title: "总量", width: 96, align: "right" as const },
  { key: "latest_used_at", title: "最近使用", width: 168, ellipsis: true },
];
const usageRows = computed<Record<string, unknown>[]>(
  () => (props.usage?.users ?? []) as unknown as Record<string, unknown>[],
);

watch(
  () => [props.sharing, visible.value] as const,
  ([sharing, isVisible]) => {
    if (!isVisible || !sharing) return;
    visibility.value = sharing.visibility;
    selectedIdentityIds.value = [...sharing.selected_identity_ids];
  },
  { immediate: true },
);

function updateVisibility(value: string | number | boolean) {
  if (typeof value !== "string") return;
  visibility.value = value as ProviderSharing["visibility"];
  if (visibility.value !== "selected") selectedIdentityIds.value = [];
}

function saveSharing() {
  emit("save-sharing", {
    visibility: visibility.value,
    identity_ids:
      visibility.value === "selected" ? [...selectedIdentityIds.value] : [],
  });
}

function close() {
  visible.value = false;
  emit("close");
}

function formatDate(value: unknown) {
  if (typeof value !== "string" || !value) return "暂无";
  return new Date(value).toLocaleString("zh-CN", {
    dateStyle: "short",
    timeStyle: "short",
  });
}
</script>

<template>
  <Drawer
    :visible="visible"
    :title="provider ? `${provider.name} · 共享与统计` : '共享与统计'"
    size="xlarge"
    :blocked="pending"
    @update:visible="
      (nextVisible) => (nextVisible ? (visible = true) : close())
    "
  >
    <div class="sharing-drawer">
      <section class="drawer-section">
        <div class="section-heading">
          <div>
            <h3>分享设置</h3>
            <p>
              创建人：{{ provider?.owner_display_name || "暂无" }}
              <Tag v-if="!provider?.can_manage" size="small">只读</Tag>
            </p>
          </div>
        </div>
        <RadioGroup
          :model-value="visibility"
          :options="visibilityOptions"
          variant="button"
          :disabled="!provider?.can_manage || pending"
          aria-label="共享范围"
          @update:model-value="updateVisibility"
        />
        <Select
          v-if="visibility === 'selected'"
          v-model="selectedIdentityIds"
          label="可见身份"
          placeholder="选择身份"
          :options="identityOptions"
          multiple
          :disabled="!provider?.can_manage || pending"
        />
        <p
          v-if="visibility === 'selected' && !identityOptions.length"
          class="empty-text"
        >
          暂无可授权的其他身份。
        </p>
      </section>

      <section class="drawer-section">
        <div class="section-heading">
          <div>
            <h3>使用统计</h3>
            <p>所有能够看到该 Provider 的身份都可查看完整统计。</p>
          </div>
          <small
            >最近使用：{{ formatDate(usage?.latest_used_at ?? null) }}</small
          >
        </div>
        <div class="usage-cards">
          <StatCard title="请求数" :value="usage?.total_requests ?? 0" />
          <StatCard title="输入 Token" :value="usage?.input_tokens ?? 0" />
          <StatCard title="输出 Token" :value="usage?.output_tokens ?? 0" />
          <StatCard title="总 Token" :value="usage?.total_tokens ?? 0" />
        </div>
        <Table
          :columns="usageColumns"
          :data="usageRows"
          empty-text="暂无使用记录"
          :loading="loading"
          row-key="identity_id"
          fixed-header
        >
          <template #cell-latest_used_at="{ row }">
            {{ formatDate(row.latest_used_at) }}
          </template>
        </Table>
      </section>
    </div>
    <template #footer>
      <Button @click="close">关闭</Button>
      <Button
        v-if="provider?.can_manage"
        variant="primary"
        :disabled="pending"
        @click="saveSharing"
      >
        {{ pending ? "保存中..." : "保存共享设置" }}
      </Button>
    </template>
  </Drawer>
</template>

<style scoped>
.sharing-drawer,
.drawer-section,
.usage-cards {
  display: grid;
  gap: var(--spacing-lg);
}

.sharing-drawer {
  min-height: 100%;
  padding: var(--spacing-lg);
}

.drawer-section {
  min-width: 0;
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
.section-heading small,
.empty-text {
  color: var(--st-text-secondary);
}

.section-heading p {
  margin-top: var(--spacing-xs);
}

.usage-cards {
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

@media (max-width: 760px) {
  .usage-cards {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
