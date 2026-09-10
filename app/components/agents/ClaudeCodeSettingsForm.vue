<script setup lang="ts">
import { Input, Select, Toggle } from "@stellar/ui";
import type { ClaudeCodeSettingsDraft } from "~/utils/agentSettings";

type SelectOption = {
  value: string;
  label: string;
  description?: string;
};

const model = defineModel<ClaudeCodeSettingsDraft>({ required: true });
const props = defineProps<{
  endpointOptions: SelectOption[];
  modelOptions: SelectOption[];
}>();

const assignableModels = computed(() => [
  { value: "", label: "不设置" },
  ...props.modelOptions,
]);

function updateNumber(
  key: "apiTimeoutMs" | "maxOutputTokens",
  value: string | number,
) {
  const text = String(value).trim();
  const parsed = Number(text);
  model.value[key] =
    text === "" || !Number.isFinite(parsed) ? null : Math.trunc(parsed);
}
</script>

<template>
  <section class="agent-settings__group">
    <div class="agent-settings__group-header"><h3>模型与接入</h3></div>
    <div class="agent-settings__fields">
      <Select
        v-model="model.endpoint"
        label="接入点"
        :options="endpointOptions"
      />
      <Select v-model="model.model" label="默认模型" :options="modelOptions" />
    </div>
  </section>
  <section class="agent-settings__group">
    <div class="agent-settings__group-header"><h3>模型分配</h3></div>
    <div class="agent-settings__fields">
      <Select
        v-model="model.opusModel"
        label="Opus 别名"
        :options="assignableModels"
      />
      <Select
        v-model="model.sonnetModel"
        label="Sonnet 别名"
        :options="assignableModels"
      />
      <Select
        v-model="model.haikuModel"
        label="Haiku 别名（含后台任务）"
        :options="assignableModels"
      />
      <Select
        v-model="model.subagentModel"
        label="子智能体模型"
        :options="assignableModels"
      />
    </div>
  </section>
  <section class="agent-settings__group">
    <div class="agent-settings__group-header"><h3>接入与输出</h3></div>
    <div class="agent-settings__rows">
      <div class="agent-settings__row">
        <span class="agent-settings__label">请求超时（毫秒）</span>
        <Input
          :model-value="model.apiTimeoutMs ?? ''"
          class="agent-settings__value"
          type="number"
          :min="1"
          placeholder="默认 600000"
          aria-label="请求超时（毫秒）"
          @update:model-value="updateNumber('apiTimeoutMs', $event)"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">最大输出 Token</span>
        <Input
          :model-value="model.maxOutputTokens ?? ''"
          class="agent-settings__value"
          type="number"
          :min="1"
          placeholder="默认 32000"
          aria-label="最大输出 Token"
          @update:model-value="updateNumber('maxOutputTokens', $event)"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">MCP 工具搜索</span>
        <Toggle v-model="model.toolSearchEnabled" aria-label="MCP 工具搜索" />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">关闭非必要流量</span>
        <Toggle
          v-model="model.nonessentialTrafficDisabled"
          aria-label="关闭非必要流量"
        />
      </div>
    </div>
  </section>
</template>

<style scoped>
.agent-settings__group {
  display: grid;
  min-width: 0;
  gap: var(--spacing-lg);
}
.agent-settings__group-header h3 {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 15px;
  font-weight: 600;
}
.agent-settings__fields {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-lg);
}
.agent-settings__rows {
  display: grid;
  gap: var(--spacing-lg);
}
.agent-settings__row {
  display: grid;
  min-height: 36px;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--spacing-xl);
}
.agent-settings__label {
  color: var(--st-text-secondary);
  font-size: 14px;
  font-weight: 400;
}
.agent-settings__value {
  width: 168px;
}
</style>
