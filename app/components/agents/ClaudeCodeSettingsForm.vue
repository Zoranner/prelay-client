<script setup lang="ts">
import { RadioGroup, Select } from "@stellar/ui";

type SelectOption = { value: string; label: string; description?: string };

type ClaudeCodeSettingsDraft = {
  endpoint: string;
  opusModel: string;
  sonnetModel: string;
  haikuModel: string;
  subagentModel: string;
  effort: string;
  language: string;
  permissionMode: string;
  rules: string;
};

const model = defineModel<ClaudeCodeSettingsDraft>({ required: true });
defineProps<{
  endpointOptions: SelectOption[];
  modelOptions: SelectOption[];
}>();

const effortOptions = [
  { value: "low", label: "低" },
  { value: "medium", label: "中" },
  { value: "high", label: "高" },
  { value: "xhigh", label: "很高" },
];
const permissionOptions = [
  { value: "manual", label: "手动确认" },
  { value: "acceptEdits", label: "允许编辑" },
  { value: "auto", label: "自动执行" },
];
const languageOptions = [
  { value: "中文", label: "中文" },
  { value: "English", label: "English" },
];
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
      <Select
        v-model="model.language"
        label="界面语言"
        :options="languageOptions"
      />
      <Select
        v-model="model.opusModel"
        label="Opus 模型"
        :options="modelOptions"
      />
      <Select
        v-model="model.sonnetModel"
        label="Sonnet 模型"
        :options="modelOptions"
      />
      <Select
        v-model="model.haikuModel"
        label="Haiku 模型"
        :options="modelOptions"
      />
      <Select
        v-model="model.subagentModel"
        label="子智能体模型"
        :options="modelOptions"
      />
    </div>
  </section>

  <section class="agent-settings__group">
    <div class="agent-settings__group-header"><h3>执行与协作</h3></div>
    <div class="agent-settings__rows">
      <div class="agent-settings__row">
        <span class="agent-settings__label">推理强度</span>
        <RadioGroup
          v-model="model.effort"
          :options="effortOptions"
          size="small"
          variant="button"
        />
      </div>
      <div class="agent-settings__row">
        <span class="agent-settings__label">工具权限</span>
        <RadioGroup
          v-model="model.permissionMode"
          :options="permissionOptions"
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
</style>
