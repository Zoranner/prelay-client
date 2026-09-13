<script setup lang="ts">
import type { ExtensionMcpPreview } from "~/stores/relay";

const props = defineProps<{
  preview: ExtensionMcpPreview;
}>();

const transport = computed(() => props.preview.manifest.transport);
const isStdio = computed(() => transport.value.type === "stdio");
const command = computed(() =>
  transport.value.type === "stdio" ? transport.value.command[0] : "",
);
const url = computed(() =>
  transport.value.type === "http" ? transport.value.url : "",
);
const argumentsList = computed<string[]>(() =>
  transport.value.type === "stdio" ? transport.value.command.slice(1) : [],
);
const headers = computed<[string, string][]>(() =>
  transport.value.type === "http"
    ? Object.entries(transport.value.headers)
    : [],
);
const environmentNames = computed<string[]>(() => {
  if (transport.value.type === "stdio") {
    return Object.keys(transport.value.environment);
  }
  return [...new Set(Object.values(transport.value.headers))];
});
</script>

<template>
  <div class="mcp-install-form">
    <section class="mcp-install-section">
      <div class="mcp-install-section__header">
        <h3 class="mcp-install-section__title">
          {{ isStdio ? "启动配置" : "连接配置" }}
        </h3>
      </div>
      <dl class="mcp-install-spec">
        <dt>{{ isStdio ? "启动命令" : "服务地址" }}</dt>
        <dd>
          <code class="mcp-install-value">{{ isStdio ? command : url }}</code>
        </dd>
        <template v-if="argumentsList.length">
          <dt>启动参数</dt>
          <dd class="mcp-install-items">
            <code
              v-for="(argument, index) in argumentsList"
              :key="index"
              class="mcp-install-value"
            >
              {{ argument }}
            </code>
          </dd>
        </template>
        <template v-if="headers.length">
          <dt>请求头</dt>
          <dd class="mcp-install-items">
            <span
              v-for="[header, variable] in headers"
              :key="header"
              class="mcp-install-mapping"
            >
              <code class="mcp-install-value">{{ header }}</code>
              <span class="mcp-install-mapping__arrow" aria-hidden="true">
                →
              </span>
              <code class="mcp-install-value">{{ variable }}</code>
            </span>
          </dd>
        </template>
      </dl>
    </section>

    <section v-if="environmentNames.length" class="mcp-install-section">
      <div class="mcp-install-section__header">
        <h3 class="mcp-install-section__title">环境变量</h3>
        <span class="mcp-install-section__count">
          {{ environmentNames.length }}
        </span>
      </div>
      <p class="mcp-install-environment__hint">
        安装只写入变量名引用，取值由本机环境变量提供。
      </p>
      <div class="mcp-install-environment">
        <code
          v-for="name in environmentNames"
          :key="name"
          class="mcp-install-environment__name"
        >
          {{ name }}
        </code>
      </div>
    </section>
  </div>
</template>

<style scoped>
.mcp-install-form {
  display: grid;
  min-width: 0;
  gap: var(--spacing-xl);
}

.mcp-install-section {
  display: grid;
  min-width: 0;
  gap: var(--spacing-md);
}

.mcp-install-section__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
}

.mcp-install-section__title {
  margin: 0;
  color: var(--st-text-primary);
  font-size: var(--text-sm);
  font-weight: 600;
}

.mcp-install-section__count {
  color: var(--st-text-muted);
  font-size: var(--text-xs);
  font-weight: 400;
}

.mcp-install-spec {
  display: grid;
  min-width: 0;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: var(--spacing-sm) var(--spacing-lg);
  margin: 0;
  padding: var(--spacing-md);
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-md);
  background: var(--st-bg-base);
}

.mcp-install-spec dt {
  color: var(--st-text-muted);
  font-size: var(--text-xs);
  line-height: 1.5rem;
}

.mcp-install-spec dd {
  min-width: 0;
  margin: 0;
}

.mcp-install-items {
  display: grid;
  gap: var(--spacing-2xs);
}

.mcp-install-value {
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--st-text-primary);
  font-family: var(--font-family-mono);
  font-size: var(--text-sm);
  line-height: 1.5rem;
}

.mcp-install-mapping {
  display: flex;
  min-width: 0;
  align-items: baseline;
  gap: var(--spacing-xs);
}

.mcp-install-mapping__arrow {
  color: var(--st-text-muted);
  font-size: var(--text-xs);
}

.mcp-install-environment {
  display: grid;
  min-width: 0;
  grid-template-columns: minmax(0, 1fr);
  gap: var(--spacing-2xs);
}

.mcp-install-environment__hint {
  margin: 0 0 var(--spacing-md);
  color: var(--st-text-muted);
  font-size: var(--text-xs);
  line-height: 1.5rem;
}

.mcp-install-environment__name {
  color: var(--st-text-secondary);
  font-family: var(--font-family-mono);
  font-size: var(--text-xs);
}
</style>
