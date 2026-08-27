<script setup lang="ts">
import { Input, RadioGroup, Select, Toggle } from "@stellar/ui";
import type { CodexSettingsDraft } from "~/utils/agentSettings";

type SelectOption = { value: string; label: string; description?: string };

const model = defineModel<CodexSettingsDraft>({ required: true });
const props = defineProps<{
  visible: boolean;
  endpointOptions: SelectOption[];
  modelOptions: SelectOption[];
  customEndpointValue: string;
}>();

const isCustomEndpoint = computed(
  () => model.value.endpoint === props.customEndpointValue,
);
const effortOptions = [
  { value: "low", label: "低" },
  { value: "medium", label: "中" },
  { value: "high", label: "高" },
  { value: "xhigh", label: "很高" },
];
const personalityOptions = [
  { value: "pragmatic", label: "务实" },
  { value: "friendly", label: "友好" },
  { value: "direct", label: "直接" },
];
const sandboxOptions = [
  { value: "read-only", label: "只读" },
  { value: "workspace-write", label: "工作区写入" },
  { value: "danger-full-access", label: "完全访问" },
];
const shellEnvironmentOptions = [
  { value: "all", label: "全部继承" },
  { value: "core", label: "仅基础环境" },
  { value: "none", label: "不继承" },
];
const windowsSandboxOptions = [
  { value: "unelevated", label: "非提升" },
  { value: "elevated", label: "提升" },
];
const codexFeatures = [
  { key: "memories", label: "长期记忆" },
  { key: "goals", label: "目标管理" },
  { key: "workspaceDependencies", label: "工作区依赖" },
] as const;

function updateNumber(
  key: "maxThreads" | "maxDepth" | "jobMaxRuntimeSeconds",
  value: string | number,
) {
  model.value[key] = Number(value);
}

watch(
  () => model.value.endpoint,
  (endpoint, previous) => {
    if (
      props.visible &&
      endpoint === props.customEndpointValue &&
      previous !== props.customEndpointValue
    ) {
      model.value.customBaseUrl = "";
      model.value.customToken = "";
    }
  },
);
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
      <template v-if="isCustomEndpoint">
        <Input v-model="model.model" label="默认模型" />
        <Input
          v-model="model.customBaseUrl"
          label="Base URL"
          placeholder="https://api.example.com/v1"
        />
        <Input v-model="model.customToken" label="Token" />
      </template>
      <Select
        v-else
        v-model="model.model"
        label="默认模型"
        :options="modelOptions"
      />
    </div>
    <div class="agent-settings__rows">
      <div class="agent-settings__row">
        <span class="agent-settings__label">推理强度</span>
        <RadioGroup
          v-model="model.reasoningEffort"
          :options="effortOptions"
          size="small"
          variant="button"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">交互风格</span>
        <RadioGroup
          v-model="model.personality"
          :options="personalityOptions"
          size="small"
          variant="button"
        />
      </div>
    </div>
  </section>

  <section class="agent-settings__group">
    <div class="agent-settings__group-header"><h3>执行与网络</h3></div>
    <div class="agent-settings__rows">
      <div class="agent-settings__row">
        <span class="agent-settings__label">执行权限</span>
        <RadioGroup
          v-model="model.sandbox"
          :options="sandboxOptions"
          size="small"
          variant="button"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">实时网络搜索</span>
        <Toggle v-model="model.webSearch" aria-label="实时网络搜索" />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">工作区网络访问</span>
        <Toggle v-model="model.networkAccess" aria-label="工作区网络访问" />
      </div>
    </div>
  </section>

  <section class="agent-settings__group">
    <div class="agent-settings__group-header"><h3>协作与记忆</h3></div>
    <div class="agent-settings__rows">
      <div class="agent-settings__row">
        <span class="agent-settings__label">最大并发智能体</span>
        <Input
          :model-value="model.maxThreads"
          class="agent-settings__value"
          type="number"
          :min="1"
          aria-label="最大并发智能体"
          @update:model-value="updateNumber('maxThreads', $event)"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">子智能体最大深度</span>
        <Input
          :model-value="model.maxDepth"
          class="agent-settings__value"
          type="number"
          :min="0"
          aria-label="子智能体最大深度"
          @update:model-value="updateNumber('maxDepth', $event)"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">单项任务时限</span>
        <Input
          :model-value="model.jobMaxRuntimeSeconds"
          class="agent-settings__value"
          type="number"
          :min="1"
          aria-label="单项任务时限（秒）"
          @update:model-value="updateNumber('jobMaxRuntimeSeconds', $event)"
        />
      </div>
      <div
        v-for="feature in codexFeatures"
        :key="feature.key"
        class="agent-settings__row"
      >
        <span class="agent-settings__label">{{ feature.label }}</span>
        <Toggle
          v-model="model.features[feature.key]"
          :aria-label="feature.label"
        />
      </div>
    </div>
  </section>

  <section class="agent-settings__group">
    <div class="agent-settings__group-header"><h3>运行环境</h3></div>
    <div class="agent-settings__rows">
      <div class="agent-settings__row">
        <span class="agent-settings__label">禁用响应存储</span>
        <Toggle
          v-model="model.disableResponseStorage"
          aria-label="禁用响应存储"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">Shell 环境继承</span>
        <Select
          v-model="model.shellEnvironmentInherit"
          class="agent-settings__value"
          aria-label="Shell 环境继承"
          :options="shellEnvironmentOptions"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">Windows 沙箱</span>
        <RadioGroup
          v-model="model.windowsSandbox"
          :options="windowsSandboxOptions"
          size="small"
          variant="button"
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
