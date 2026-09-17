<script setup lang="ts">
import {
  Badge,
  Button,
  Icon,
  Loading,
  Drawer,
  Select,
  useConfirm,
  useNotification,
} from "@stellar/ui";
import type {
  AgentClient,
  ExtensionCatalogPackage,
  ExtensionMcpPreview,
} from "~/stores/relay";
import McpInstallForm from "~/components/extensions/McpInstallForm.vue";
import { agentSectionOptions } from "~/utils/agentClient";
import { mcpPreviewMatchesExtension } from "~/utils/extensionMcpPreview";
import { synchronizeExtensionInstallSelection } from "~/utils/extensionInstallSelection";
import { errorText, toRelayError } from "~/utils/errors";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionCatalogPackage | null;
  detectedClients: AgentClient[];
}>();
const emit = defineEmits<{
  installed: [kind: ExtensionCatalogPackage["kind"]];
  refreshed: [kind: ExtensionCatalogPackage["kind"]];
}>();
const { invokeLocalCommand } = useLocalCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
const selectedClients = ref<AgentClient[]>([]);
const installing = ref(false);
const mcpPreview = ref<ExtensionMcpPreview | null>(null);
const mcpPreviewLoading = ref(false);
const mcpPreviewError = ref("");
let mcpPreviewRequest = 0;
const actionLabel = computed(() =>
  props.extension?.installAction === "update"
    ? "更新"
    : props.extension?.installAction === "partial"
      ? "补装"
      : "安装",
);

const detected = computed(() => new Set(props.detectedClients));
const isMcp = computed(() => props.extension?.kind === "mcp");
const kindIcon = computed(() => {
  const kind = props.extension?.kind;
  if (!kind) return "";
  const section = kind === "rule" ? "rules" : kind;
  return (
    agentSectionOptions.find((option) => option.value === section)?.icon ?? ""
  );
});
const identityName = computed(() =>
  isMcp.value
    ? (mcpPreview.value?.manifest.name ?? "")
    : (props.extension?.name ?? ""),
);
const identityVersion = computed(() =>
  isMcp.value
    ? (mcpPreview.value?.version ?? "")
    : (props.extension?.version ?? ""),
);
const transportLabel = computed(() => {
  if (!isMcp.value || !mcpPreview.value) return "";
  return mcpPreview.value.manifest.transport.type === "stdio"
    ? "STDIO"
    : "HTTP";
});
const supportedDetectedClients = computed(() => props.detectedClients);
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
    detected: supportedDetectedClients.value,
    kind: props.extension.kind,
    next: values,
    previous: selectedClients.value,
  });
}

async function install(overwrite = false) {
  if (!props.extension || !selectedClients.value.length) return;
  if (
    props.extension.kind === "mcp" &&
    (!mcpPreview.value ||
      !mcpPreviewMatchesExtension(mcpPreview.value, props.extension))
  ) {
    return;
  }
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
    emit("installed", props.extension.kind);
  } catch (caught) {
    const error = caught as { code?: string; message?: string };
    if (!overwrite && error.code === "extension_target_exists") {
      const confirmed = await confirmAction({
        title:
          props.extension.kind === "mcp" ? "覆盖已有 MCP 配置" : "覆盖已有技能",
        message:
          props.extension.kind === "mcp"
            ? "目标 MCP 服务名已存在，是否覆盖安装？"
            : "目标技能目录已存在，是否覆盖安装？",
        description:
          props.extension.kind === "mcp"
            ? "覆盖会替换目标 MCP 服务配置，并更新当前扩展包的版本记录。"
            : "覆盖会删除该技能目录中的现有文件，然后写入当前扩展包内容。",
        confirmText: "覆盖",
        danger: true,
      });
      if (confirmed) {
        await install(true);
      }
      return;
    }
    if (props.extension?.kind === "mcp") {
      emit("refreshed", props.extension.kind);
    }
    notifications.error(errorText(toRelayError(caught)), {
      title: "扩展安装失败",
    });
  } finally {
    installing.value = false;
  }
}

watch(
  () =>
    [
      visible.value,
      props.extension?.name,
      props.extension?.version,
      props.extension?.commitSha,
    ] as const,
  async ([isVisible]) => {
    const request = ++mcpPreviewRequest;
    if (!isVisible) return;
    const extension = props.extension;
    mcpPreview.value = null;
    mcpPreviewError.value = "";
    selectedClients.value = ["partial", "update"].includes(
      extension?.installAction ?? "",
    )
      ? supportedDetectedClients.value.filter((client) =>
          extension?.installAction === "partial"
            ? !extension.installedClients.includes(client)
            : extension?.installedClients.includes(client),
        )
      : [...supportedDetectedClients.value];
    if (extension?.kind !== "mcp") return;
    mcpPreviewLoading.value = true;
    try {
      const preview = await invokeLocalCommand<ExtensionMcpPreview>(
        "extensions_mcp_preview",
        { package: extension },
        { notify: false, trackPending: false },
      );
      if (
        request === mcpPreviewRequest &&
        visible.value &&
        props.extension &&
        mcpPreviewMatchesExtension(preview, props.extension)
      ) {
        mcpPreview.value = preview;
      }
    } catch (caught) {
      if (request === mcpPreviewRequest) {
        mcpPreviewError.value = errorText(toRelayError(caught));
      }
    } finally {
      if (request === mcpPreviewRequest) mcpPreviewLoading.value = false;
    }
  },
);
</script>

<template>
  <Drawer
    :visible="visible"
    :title="`${actionLabel}扩展`"
    position="right"
    size="xlarge"
    :blocked="installing"
    @update:visible="(nextVisible) => (visible = nextVisible)"
  >
    <div class="extension-install-drawer">
      <header v-if="identityName" class="extension-install-drawer__identity">
        <span
          v-if="kindIcon"
          class="extension-install-drawer__identity-icon"
          aria-hidden="true"
        >
          <Icon :icon="kindIcon" size="16" />
        </span>
        <div class="extension-install-drawer__identity-body">
          <h2 class="extension-install-drawer__identity-name">
            {{ identityName }}
          </h2>
          <span
            v-if="identityVersion"
            class="extension-install-drawer__identity-version"
          >
            {{ identityVersion }}
          </span>
        </div>
        <Badge
          v-if="transportLabel"
          class="extension-install-drawer__identity-transport"
          semantic="info"
          variant="soft"
        >
          {{ transportLabel }}
        </Badge>
      </header>
      <div class="extension-install-drawer__target">
        <Select
          :model-value="selectedClients"
          :options="clientOptions"
          label="安装到智能体"
          placeholder="选择智能体"
          multiple
          @update:model-value="selectClients"
        />
      </div>
      <div v-if="isMcp" class="extension-install-drawer__preview">
        <Loading v-if="mcpPreviewLoading" visible text="正在读取 MCP 配置..." />
        <McpInstallForm v-else-if="mcpPreview" :preview="mcpPreview" />
        <p v-else class="extension-install-drawer__error">
          {{ mcpPreviewError }}
        </p>
      </div>
    </div>
    <template #footer>
      <div class="extension-install-drawer__footer">
        <div class="extension-install-drawer__actions">
          <Button :disabled="installing" @click="visible = false">
            取消
          </Button>
          <Button
            semantic="primary"
            variant="solid"
            :disabled="
              installing || !selectedClients.length || (isMcp && !mcpPreview)
            "
            @click="install()"
          >
            {{ installing ? `${actionLabel}中...` : actionLabel }}
          </Button>
        </div>
      </div>
    </template>
  </Drawer>
</template>

<style scoped>
.extension-install-drawer {
  display: flex;
  height: 100%;
  min-height: 0;
  min-width: 0;
  flex-direction: column;
  gap: var(--spacing-xl);
  padding: var(--spacing-lg);
}

.extension-install-drawer__target {
  flex: 0 0 auto;
  min-width: 0;
}

.extension-install-drawer__identity {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-md);
}

.extension-install-drawer__identity-icon {
  display: flex;
  width: 32px;
  height: 32px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-md);
  background: var(--st-bg-surface);
  color: var(--st-text-secondary);
}

.extension-install-drawer__identity-body {
  display: grid;
  min-width: 0;
  gap: var(--spacing-2xs);
}

.extension-install-drawer__identity-name {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  color: var(--st-text-primary);
  font-size: 16px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.extension-install-drawer__identity-version {
  min-width: 0;
  overflow: hidden;
  color: var(--st-text-muted);
  font-family: var(--font-family-mono);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.extension-install-drawer__identity-transport {
  flex: 0 0 auto;
  margin-left: auto;
}

.extension-install-drawer__preview {
  min-height: 0;
  min-width: 0;
  flex: 1 1 auto;
  overflow-y: auto;
}

.extension-install-drawer__footer {
  display: flex;
  justify-content: flex-end;
  width: 100%;
}

.extension-install-drawer__actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

.extension-install-drawer__error {
  margin: 0;
  color: var(--st-semantic-error);
  font-size: var(--text-sm);
}

@media (max-width: 720px) {
  .extension-install-drawer {
    padding: var(--spacing-md);
  }

  .extension-install-drawer__footer {
    justify-content: stretch;
  }

  .extension-install-drawer__actions {
    justify-content: flex-end;
  }
}
</style>
