<script setup lang="ts">
import { Button, Modal, Select } from "@stellar/ui";
import type { AgentClient, ExtensionCatalogPackage } from "~/stores/relay";
import { synchronizeExtensionInstallSelection } from "~/utils/extensionInstallSelection";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionCatalogPackage | null;
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
  if (!props.extension) return;
  selectedClients.value = synchronizeExtensionInstallSelection({
    detected: props.detectedClients,
    kind: props.extension.kind,
    next: values,
    previous: selectedClients.value,
  });
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
    :title="extension ? `安装 ${extension.name}` : '安装扩展'"
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
        label="安装到智能体"
        placeholder="选择智能体"
        multiple
        @update:model-value="selectClients"
      />
    </div>
    <template #footer>
      <Button :disabled="installing" @click="visible = false">取消</Button>
      <Button
        variant="primary"
        :disabled="installing || !selectedClients.length"
        @click="install"
      >
        {{ installing ? "安装中..." : "安装" }}
      </Button>
    </template>
  </Modal>
</template>

<style scoped>
.extension-install {
  position: relative;
}
</style>
