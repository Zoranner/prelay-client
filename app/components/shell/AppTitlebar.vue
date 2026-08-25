<script setup lang="ts">
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "@stellar/ui";
import prelayIcon from "~/assets/images/prelay-icon.png";

const isMaximized = ref(false);
const desktopPreferencesDialog = useDesktopPreferencesDialog();
let unlistenResized: UnlistenFn | undefined;

const isTauriRuntime = () => "__TAURI_INTERNALS__" in globalThis;

async function withWindow<T>(
  action: (window: ReturnType<typeof getCurrentWindow>) => Promise<T>,
) {
  if (!isTauriRuntime()) return undefined;
  try {
    return await action(getCurrentWindow());
  } catch {
    return undefined;
  }
}

async function refreshMaximizedState() {
  const maximized = await withWindow((window) => window.isMaximized());
  if (typeof maximized === "boolean") isMaximized.value = maximized;
}

const minimize = () => withWindow((window) => window.minimize());
const openDesktopPreferences = () => desktopPreferencesDialog.open();

async function toggleMaximize() {
  await withWindow((window) => window.toggleMaximize());
  await refreshMaximizedState();
}

const closeWindow = () => withWindow((window) => window.close());

onMounted(async () => {
  await refreshMaximizedState();
  await withWindow(async (window) => {
    unlistenResized = await window.onResized(() => {
      void refreshMaximizedState();
    });
  });
});

onUnmounted(() => unlistenResized?.());
</script>

<template>
  <header
    class="app-titlebar"
    data-tauri-drag-region
    @dblclick="toggleMaximize"
  >
    <div class="brand" data-tauri-drag-region>
      <span class="brand-mark" data-tauri-drag-region>
        <img :src="prelayIcon" alt="" data-tauri-drag-region />
      </span>
      <div class="brand-copy" data-tauri-drag-region>
        <strong data-tauri-drag-region>Prelay</strong>
      </div>
    </div>

    <div class="window-actions" @dblclick.stop>
      <Button
        square
        size="small"
        variant="ghost"
        icon="ph:gear-six"
        aria-label="设置"
        title="设置"
        @click="openDesktopPreferences"
      />
      <Button
        square
        size="small"
        variant="ghost"
        icon="ph:minus"
        aria-label="最小化"
        title="最小化"
        @click="minimize"
      />
      <Button
        square
        size="small"
        variant="ghost"
        :icon="isMaximized ? 'ph:arrows-in' : 'ph:arrows-out'"
        :aria-label="isMaximized ? '还原' : '最大化'"
        :title="isMaximized ? '还原' : '最大化'"
        @click="toggleMaximize"
      />
      <Button
        class="window-action-close"
        square
        size="small"
        variant="ghost"
        icon="ph:x"
        aria-label="关闭"
        title="关闭"
        @click="closeWindow"
      />
    </div>
  </header>
</template>

<style scoped>
.app-titlebar {
  display: grid;
  height: var(--pr-titlebar-height);
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  border-bottom: 1px solid var(--st-border-divider);
  background: linear-gradient(
    to bottom,
    var(--st-bg-elevated),
    var(--st-bg-header)
  );
  box-shadow: var(--shadow-highlight-xs);
  user-select: none;
  -webkit-app-region: drag;
}

.brand,
.brand-copy,
.window-actions {
  display: flex;
  align-items: center;
}

.brand {
  min-width: 0;
  gap: 10px;
  padding: 0 14px;
}

.brand-mark {
  display: block;
  width: 24px;
  height: 24px;
  flex: 0 0 auto;
}

.brand-mark img {
  display: block;
  width: 100%;
  height: 100%;
  border-radius: var(--radius-md);
}

.brand-copy {
  min-width: 0;
  gap: 12px;
}

.brand-copy strong {
  color: var(--st-text-primary);
  font-size: 13px;
}

.brand-copy span {
  overflow: hidden;
  color: var(--st-text-muted);
  font-family: var(--font-family-mono);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.window-actions {
  height: 100%;
  gap: 2px;
  padding-right: 8px;
  -webkit-app-region: no-drag;
}

.window-actions :deep(button) {
  width: 34px;
  height: 28px;
  border-color: transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  box-shadow: none;
}

.window-actions :deep(button:hover) {
  background: var(--st-bg-surface);
}

.window-actions :deep(.window-action-close:hover) {
  color: #fff;
  background: var(--st-danger);
}
</style>
