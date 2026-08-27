<script setup lang="ts">
import { Drawer, Loading, Tag } from "@stellar/ui";
import type { ExtensionPackage } from "~/stores/relay";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionPackage | null;
}>();
const { invokeLocalCommand } = useLocalCommand();
const readme = ref("");
const loading = ref(false);

const kindLabel = computed(() => {
  const kind = props.extension?.kind;
  return { rule: "规则", plugin: "插件", mcp: "MCP", skill: "Skill" }[kind ?? "skill"];
});

watch(
  () => [visible.value, props.extension?.repository, props.extension?.commitSha] as const,
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
    :title="extension?.name ?? '扩展详情'"
    size="large"
    :show-footer="false"
    :blocked="loading"
    @update:visible="(nextVisible) => (visible = nextVisible)"
  >
    <div v-if="extension" class="extension-detail">
      <div class="extension-detail__meta">
        <Tag size="small">{{ kindLabel }}</Tag>
        <span>{{ extension.version }}</span>
        <span>风险：{{ extension.risk }}</span>
      </div>
      <Loading v-if="loading" visible text="正在读取说明..." />
      <pre v-else class="extension-detail__readme">{{ readme }}</pre>
    </div>
  </Drawer>
</template>

<style scoped>
.extension-detail {
  display: flex;
  position: relative;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
}

.extension-detail__meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--st-text-secondary);
  font-size: 13px;
}

.extension-detail__readme {
  margin: 0;
  min-height: 0;
  flex: 1;
  overflow: auto;
  color: var(--st-text-primary);
  font: inherit;
  line-height: 1.65;
  white-space: pre-wrap;
}
</style>
