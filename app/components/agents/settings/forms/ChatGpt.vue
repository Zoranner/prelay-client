<script setup lang="ts">
import CodexSettingsForm from "~/components/agents/settings/forms/Codex.vue";
import type { CatalogLanguageModelResponse } from "~/types/relay";
import type { ChatGptSettingsDraft } from "~/utils/agent/settings";

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
}>();
</script>

<template>
  <CodexSettingsForm
    v-model="model"
    :visible="visible"
    :endpoint-options="endpointOptions"
    :model-options="modelOptions"
  />
</template>
