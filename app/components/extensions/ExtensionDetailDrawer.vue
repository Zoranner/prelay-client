<script setup lang="ts">
import { Button, Drawer, Loading, MarkdownViewer } from "@stellar/ui";
import type { ExtensionCatalogPackage } from "~/stores/relay";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionCatalogPackage | null;
}>();
const { invokeLocalCommand } = useLocalCommand();
const readme = ref("");
const loading = ref(false);

watch(
  () =>
    [
      visible.value,
      props.extension?.repository,
      props.extension?.commitSha,
    ] as const,
  async ([isVisible]) => {
    if (!isVisible || !props.extension) return;
    loading.value = true;
    readme.value = "";
    try {
      readme.value = await invokeLocalCommand<string>("extension_readme", {
        package: props.extension,
      });
    } catch {
      readme.value = "无法读取 README。";
    } finally {
      loading.value = false;
    }
  },
);
</script>

<template>
  <Drawer
    :visible="visible"
    title="扩展详情"
    size="large"
    :blocked="loading"
    @update:visible="(nextVisible) => (visible = nextVisible)"
  >
    <div v-if="extension" class="extension-detail">
      <Loading v-if="loading" visible text="正在读取说明..." />
      <MarkdownViewer
        v-else
        :content="readme"
        class="extension-detail__readme"
      />
    </div>
    <template #footer>
      <Button @click="visible = false">关闭</Button>
    </template>
  </Drawer>
</template>

<style scoped>
.extension-detail {
  display: flex;
  position: relative;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--spacing-lg);
}

.extension-detail__readme {
  min-height: 0;
  flex: 1;
  overflow: auto;
  color: var(--st-text-primary);
  font: inherit;
  line-height: 1.65;
}

.extension-detail__readme :deep(h1) {
  margin-top: 0;
  margin-bottom: var(--spacing-md);
}

.extension-detail__readme :deep(h2) {
  margin-top: var(--spacing-lg);
  margin-bottom: var(--spacing-sm);
}

.extension-detail__readme :deep(p),
.extension-detail__readme :deep(ul),
.extension-detail__readme :deep(ol) {
  margin-top: 0;
  margin-bottom: var(--spacing-md);
}
</style>
