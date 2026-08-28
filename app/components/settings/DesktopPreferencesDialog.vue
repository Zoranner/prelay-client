<script setup lang="ts">
import { Button, Modal, RadioGroup, Toggle } from "@stellar/ui";
import type { DesktopPreferences } from "~/composables/useDesktopPreferences";

const visible = defineModel<boolean>("visible", { default: false });
const desktopPreferences = useDesktopPreferences();
const workspaceExit = useWorkspaceExitGuard();
const draft = reactive<DesktopPreferences>({
  ...desktopPreferences.preferences.value,
});
const loading = ref(false);
const saving = ref(false);
const savedDraft = ref("");
let exitRegistration: ReturnType<typeof workspaceExit.register> | undefined;
const themeOptions = [
  { label: "跟随系统", value: "system", icon: "ph:desktop" },
  { label: "浅色", value: "light", icon: "ph:sun" },
  { label: "深色", value: "dark", icon: "ph:moon" },
];

const isDirty = computed(() => JSON.stringify(draft) !== savedDraft.value);

function closeImmediately() {
  visible.value = false;
}

async function requestClose() {
  if (exitRegistration) await exitRegistration.requestExit();
  else closeImmediately();
}

watch(visible, async (isVisible) => {
  if (!isVisible) {
    desktopPreferences.applyTheme(desktopPreferences.preferences.value.theme);
    exitRegistration?.unregister();
    exitRegistration = undefined;
    return;
  }

  exitRegistration = workspaceExit.register({
    close: closeImmediately,
    state: () =>
      loading.value || saving.value
        ? "blocked"
        : isDirty.value
          ? "discard"
          : "allow",
  });
  loading.value = true;
  try {
    const preferences = await desktopPreferences.load();
    Object.assign(draft, preferences);
    savedDraft.value = JSON.stringify(preferences);
  } finally {
    loading.value = false;
  }
});

watch(
  () => draft.theme,
  (theme) => {
    if (visible.value) desktopPreferences.applyTheme(theme);
  },
);

async function save() {
  saving.value = true;
  try {
    await desktopPreferences.save({ ...draft });
    savedDraft.value = JSON.stringify(draft);
    closeImmediately();
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Modal
    :visible="visible"
    title="设置"
    size="large"
    :blocked="loading || saving || isDirty"
    :show-cancel="false"
    :show-confirm="false"
    @update:visible="
      (nextVisible) => (nextVisible ? (visible = true) : void requestClose())
    "
  >
    <div class="desktop-preferences-dialog">
      <div class="preferences-item preferences-item--theme">
        <div class="preferences-item__copy">
          <strong>外观主题</strong>
          <span>不影响服务端配置或其他设备。</span>
        </div>
        <RadioGroup
          v-model="draft.theme"
          variant="button"
          size="small"
          :options="themeOptions"
        />
      </div>

      <div class="preferences-item">
        <div class="preferences-item__copy">
          <strong>开机自启</strong>
          <span>登录系统后自动启动 Prelay。</span>
        </div>
        <Toggle v-model="draft.autostartEnabled" aria-label="开机自启" />
      </div>
      <div
        class="preferences-item"
        :class="{ 'preferences-item--disabled': !draft.autostartEnabled }"
      >
        <div class="preferences-item__copy">
          <strong>静默启动</strong>
          <span>仅在开机自启时隐藏主窗口并保留托盘图标。</span>
        </div>
        <Toggle
          v-model="draft.silentStart"
          :disabled="!draft.autostartEnabled"
          aria-label="静默启动"
        />
      </div>
      <div class="preferences-item">
        <div class="preferences-item__copy">
          <strong>最小化到托盘</strong>
          <span>关闭主窗口后继续在系统托盘中运行。</span>
        </div>
        <Toggle
          v-model="draft.minimizeToTray"
          aria-label="关闭时最小化到托盘"
        />
      </div>
    </div>

    <template #footer>
      <Button :disabled="saving" @click="requestClose">取消</Button>
      <Button variant="primary" :disabled="saving" @click="save">保存</Button>
    </template>
  </Modal>
</template>

<style scoped>
.desktop-preferences-dialog {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.preferences-item strong,
.preferences-item span {
  margin: 0;
}

.preferences-item strong {
  color: var(--st-text-primary);
  font-size: 14px;
  font-weight: 600;
}

.preferences-item span {
  color: var(--st-text-muted);
  font-size: 13px;
}

.preferences-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  min-height: 64px;
  align-items: center;
  gap: 24px;
  padding: 10px 0;
}

.preferences-item__copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 4px;
}

.preferences-item--theme {
  min-height: 56px;
  padding: 8px 0;
}

.preferences-item--disabled .preferences-item__copy {
  opacity: 0.55;
}

@media (max-width: 620px) {
  .preferences-item {
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
  }

  .preferences-item > :last-child {
    justify-self: start;
  }
}
</style>
