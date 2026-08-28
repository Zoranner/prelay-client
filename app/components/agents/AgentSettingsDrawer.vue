<script setup lang="ts">
import { Button, Drawer } from "@stellar/ui";
import type { AgentClient } from "~/stores/relay";
import type { AgentConfiguration } from "~/utils/agentSettings";
import ChatGptSettingsForm from "~/components/agents/ChatGptSettingsForm.vue";
import CodexSettingsForm from "~/components/agents/CodexSettingsForm.vue";
import OpenCodeSettingsForm from "~/components/agents/OpenCodeSettingsForm.vue";

type SelectOption = { description?: string; label: string; value: string };

defineProps<{
  activeClient: AgentClient;
  blocked: boolean;
  customEndpointValue: string;
  endpointOptions: SelectOption[];
  modelOptions: SelectOption[];
  pending: boolean;
  visible: boolean;
}>();

const draft = defineModel<AgentConfiguration>("draft", { required: true });
const emit = defineEmits<{
  close: [];
  save: [];
  "update:visible": [visible: boolean];
}>();
</script>

<template>
  <Drawer
    :visible="visible"
    title="智能体设置"
    size="xlarge"
    :blocked="blocked"
    @update:visible="emit('update:visible', $event)"
  >
    <form
      id="agent-settings-form"
      class="agent-settings-form"
      @submit.prevent="emit('save')"
    >
      <CodexSettingsForm
        v-if="activeClient === 'codexCli'"
        v-model="draft.codexCli"
        :visible="visible"
        :endpoint-options="endpointOptions"
        :model-options="modelOptions"
        :custom-endpoint-value="customEndpointValue"
      />
      <ChatGptSettingsForm
        v-else-if="activeClient === 'chatgpt'"
        v-model="draft.chatgpt"
        :visible="visible"
        :endpoint-options="endpointOptions"
        :model-options="modelOptions"
        :custom-endpoint-value="customEndpointValue"
      />
      <OpenCodeSettingsForm
        v-else
        v-model="draft.openCode"
        :endpoint-options="endpointOptions"
        :model-options="modelOptions"
      />
    </form>
    <template #footer>
      <Button :disabled="pending" @click="emit('close')">取消</Button>
      <Button
        form="agent-settings-form"
        type="submit"
        variant="primary"
        :disabled="pending"
      >
        {{ pending ? "保存中..." : "保存" }}
      </Button>
    </template>
  </Drawer>
</template>

<style scoped>
.agent-settings-form {
  display: grid;
  gap: var(--spacing-xl);
  padding: var(--spacing-lg);
}
</style>
