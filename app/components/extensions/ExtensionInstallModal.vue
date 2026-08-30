<script setup lang="ts">
import {
  Button,
  Modal,
  Select,
  useConfirm,
  useNotification,
} from "@stellar/ui";
import type { AgentClient, ExtensionCatalogPackage } from "~/stores/relay";
import { synchronizeExtensionInstallSelection } from "~/utils/extensionInstallSelection";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionCatalogPackage | null;
  detectedClients: AgentClient[];
}>();
const emit = defineEmits<{ installed: [] }>();
const { invokeLocalCommand } = useLocalCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
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

async function install(overwrite = false) {
  if (!props.extension || !selectedClients.value.length) return;
  installing.value = true;
  try {
    await invokeLocalCommand(
      "extensions_install",
      {
        request: {
          package: props.extension,
          clients: selectedClients.value,
          overwrite,
        },
      },
      { notify: false },
    );
    visible.value = false;
    emit("installed");
  } catch (caught) {
    const error = caught as { code?: string; message?: string };
    if (!overwrite && error.code === "extension_target_exists") {
      const confirmed = await confirmAction({
        title: "覆盖已有技能",
        message: "目标技能目录已存在，是否覆盖安装？",
        description:
          "覆盖会删除该技能目录中的现有文件，然后写入当前扩展包内容。",
        confirmText: "覆盖",
        danger: true,
      });
      if (confirmed) {
        await install(true);
      }
      return;
    }
    notifications.danger(error.message ?? "扩展安装失败。", {
      title: "扩展安装失败",
    });
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
        @click="install()"
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
