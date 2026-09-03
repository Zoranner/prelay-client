<script setup lang="ts">
import CodexSettingsForm from "~/components/agents/CodexSettingsForm.vue";
import type { CatalogLanguageModelResponse } from "~/stores/relay";
import type { ChatGptSettingsDraft } from "~/utils/agentSettings";

type SelectOption = {
  value: string;
  label: string;
  description?: string;
  catalogModel?: CatalogLanguageModelResponse;
};

const model = defineModel<ChatGptSettingsDraft>({ required: true });
defineProps<{
  visible: boolean;
  endpointOptions: SelectOption[];
  modelOptions: SelectOption[];
  customEndpointValue: string;
}>();
</script>

<template>
  <CodexSettingsForm
    v-model="model"
    :visible="visible"
    :endpoint-options="endpointOptions"
    :model-options="modelOptions"
    :custom-endpoint-value="customEndpointValue"
  />
</template>
