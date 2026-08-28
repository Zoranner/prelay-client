<script setup lang="ts">
import { Select } from "@stellar/ui";
import type { OpenCodeSettingsDraft } from "~/utils/agentSettings";

type SelectOption = { value: string; label: string; description?: string };

const model = defineModel<OpenCodeSettingsDraft>({ required: true });
defineProps<{
  endpointOptions: SelectOption[];
  modelOptions: SelectOption[];
}>();
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
</style>
