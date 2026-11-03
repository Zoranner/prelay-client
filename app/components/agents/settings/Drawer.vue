<script setup lang="ts">
defineOptions({ name: "AgentSettingsDrawer" });

import { Button, Drawer } from "@stellar/ui";
import type { AgentClient } from "~/types/relay";
import type { AgentConfiguration } from "~/utils/agent/settings";
import ChatGptSettingsForm from "~/components/agents/settings/forms/ChatGpt.vue";
import CodexSettingsForm from "~/components/agents/settings/forms/Codex.vue";

type SelectOption = { description?: string; label: string; value: string };

defineProps<{
  activeClient: AgentClient;
  blocked: boolean;
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
      />
      <ChatGptSettingsForm
        v-else
        v-model="draft.chatgpt"
        :visible="visible"
        :endpoint-options="endpointOptions"
        :model-options="modelOptions"
      />
    </form>
    <template #footer>
      <Button :disabled="pending" @click="emit('close')">取消</Button>
      <Button
        form="agent-settings-form"
        type="submit"
        semantic="primary"
        variant="solid"
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
