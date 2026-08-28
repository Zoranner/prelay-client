<script setup lang="ts">
import { Button, Modal, Select } from "@stellar/ui";
import type { AgentClient, ExtensionPackage } from "~/stores/relay";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionPackage | null;
  detectedClients: AgentClient[];
}>();
const emit = defineEmits<{ installed: [] }>();
const { invokeLocalCommand } = useLocalCommand();
const selectedClients = ref<AgentClient[]>([]);
const installing = ref(false);

const detected = computed(() => new Set(props.detectedClients));
const clientOptions = computed(() => [
  {
    value: "codexCli",
    label: "Codex CLI",
    disabled: !detected.value.has("codexCli"),
  },
  {
    value: "chatgpt",
    label: "ChatGPT",
    disabled: !detected.value.has("chatgpt"),
  },
  {
    value: "openCode",
    label: "OpenCode",
    disabled: !detected.value.has("openCode"),
  },
]);

function selectClients(values: AgentClient[]) {
  const next = new Set(values);
  const previous = new Set(selectedClients.value);
  const codexChanged = next.has("codexCli") !== previous.has("codexCli");
  const chatgptChanged = next.has("chatgpt") !== previous.has("chatgpt");
  const nextClients: AgentClient[] = values.filter(
    (client) => client !== "codexCli" && client !== "chatgpt",
  );

  if (codexChanged || chatgptChanged) {
    const selectCodexHost = codexChanged
      ? next.has("codexCli")
      : next.has("chatgpt");
    if (selectCodexHost) {
      if (detected.value.has("codexCli")) nextClients.push("codexCli");
      if (detected.value.has("chatgpt")) nextClients.push("chatgpt");
    }
  } else {
    if (next.has("codexCli")) nextClients.push("codexCli");
    if (next.has("chatgpt")) nextClients.push("chatgpt");
  }

  selectedClients.value = nextClients;
}

async function install() {
  if (!props.extension || !selectedClients.value.length) return;
  installing.value = true;
  try {
    await invokeLocalCommand("extensions_install", {
      request: { package: props.extension, clients: selectedClients.value },
    });
    visible.value = false;
    emit("installed");
  } finally {
    installing.value = false;
  }
}

watch(
  () => visible.value,
  (isVisible) => {
    if (!isVisible) return;
    selectedClients.value = [...props.detectedClients];
  },
);
</script>

<template>
  <Modal
    :visible="visible"
    :title="extension ? `安装${extension.repository}` : '安装扩展'"
    size="large"
    :blocked="installing"
    :show-cancel="false"
    :show-confirm="false"
    @update:visible="(nextVisible) => (visible = nextVisible)"
  >
    <div class="extension-install">
      <Select
        :model-value="selectedClients"
        :options="clientOptions"
        label="安装到"
        placeholder="选择客户端"
        multiple
        @update:model-value="selectClients"
      />
    </div>
    <template #footer>
      <Button :disabled="installing" @click="visible = false">取消</Button>
      <Button
        variant="primary"
        :disabled="
          installing || !selectedClients.length
        "
        @click="install"
      >
        {{ installing ? '安装中...' : '安装' }}
      </Button>
    </template>
  </Modal>
</template>

<style scoped>
.extension-install {
  position: relative;
}
</style>
