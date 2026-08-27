<script setup lang="ts">
import { Button, Checkbox, Loading, Modal } from "@stellar/ui";
import type {
  AgentClient,
  ExtensionInstallPreview,
  ExtensionPackage,
} from "~/stores/relay";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionPackage | null;
  detectedClients: AgentClient[];
}>();
const emit = defineEmits<{ installed: [] }>();
const { invokeLocalCommand } = useLocalCommand();
const selected = reactive<Record<AgentClient, boolean>>({
  codexCli: false,
  chatgpt: false,
  claudeCode: false,
});
const preview = ref<ExtensionInstallPreview | null>(null);
const loadingPreview = ref(false);
const installing = ref(false);

const detected = computed(() => new Set(props.detectedClients));
const selectedClients = computed(() =>
  props.detectedClients.filter((client) => selected[client]),
);
const codexHostSelected = computed({
  get: () => selected.codexCli || selected.chatgpt,
  set: (value: boolean) => {
    selected.codexCli = value && detected.value.has("codexCli");
    selected.chatgpt = value && detected.value.has("chatgpt");
  },
});

async function loadPreview() {
  if (!props.extension) return;
  loadingPreview.value = true;
  preview.value = null;
  try {
    preview.value = await invokeLocalCommand<ExtensionInstallPreview>(
      "extension_install_preview",
      {
        request: {
          package: props.extension,
          clients: selectedClients.value,
        },
      },
      { notify: false },
    );
  } finally {
    loadingPreview.value = false;
  }
}

async function install() {
  if (!props.extension || !preview.value?.supported || !selectedClients.value.length) return;
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
    selected.codexCli = detected.value.has("codexCli");
    selected.chatgpt = detected.value.has("chatgpt");
    selected.claudeCode = detected.value.has("claudeCode");
    void loadPreview();
  },
);

watch(selected, () => {
  if (visible.value) void loadPreview();
});
</script>

<template>
  <Modal
    :visible="visible"
    :title="extension ? `安装${extension.name}` : '安装扩展'"
    size="large"
    :blocked="loadingPreview || installing"
    :show-cancel="false"
    :show-confirm="false"
    @update:visible="(nextVisible) => (visible = nextVisible)"
  >
    <div class="extension-install">
      <div class="extension-install__targets">
        <Checkbox
          v-model="codexHostSelected"
          label="Codex CLI"
          description="与 ChatGPT 共享本机落点"
          :disabled="!detected.has('codexCli')"
        />
        <Checkbox
          v-model="selected.chatgpt"
          label="ChatGPT"
          description="与 Codex CLI 共享本机落点"
          :disabled="!detected.has('chatgpt')"
        />
        <Checkbox
          v-model="selected.claudeCode"
          label="Claude Code"
          :disabled="!detected.has('claudeCode')"
        />
      </div>
      <Loading v-if="loadingPreview" visible text="正在生成安装变更..." />
      <p v-else-if="preview?.message" class="extension-install__message">
        {{ preview.message }}
      </p>
      <ul v-else class="extension-install__actions">
        <li v-for="action in preview?.actions" :key="action.target">
          <span>{{ action.description }}</span>
          <code>{{ action.target }}</code>
        </li>
      </ul>
    </div>
    <template #footer>
      <Button :disabled="installing" @click="visible = false">取消</Button>
      <Button
        variant="primary"
        :disabled="
          loadingPreview ||
          installing ||
          !preview?.supported ||
          !selectedClients.length
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
  display: grid;
  position: relative;
  min-height: 180px;
  gap: var(--spacing-lg);
}

.extension-install__targets {
  display: grid;
  gap: var(--spacing-sm);
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--st-border-divider);
}

.extension-install__message {
  margin: 0;
  color: var(--st-text-secondary);
}

.extension-install__actions {
  display: grid;
  margin: 0;
  padding: 0;
  gap: var(--spacing-sm);
  list-style: none;
}

.extension-install__actions li {
  display: grid;
  gap: 4px;
}

.extension-install__actions span {
  color: var(--st-text-primary);
}

.extension-install__actions code {
  overflow: hidden;
  color: var(--st-text-secondary);
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
