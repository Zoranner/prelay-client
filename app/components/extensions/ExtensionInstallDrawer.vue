<script setup lang="ts">
import {
  Button,
  Icon,
  Loading,
  Drawer,
  Select,
  Tag,
  useConfirm,
  useNotification,
} from "@stellar/ui";
import type {
  AgentClient,
  ExtensionCatalogPackage,
  ExtensionInstallResult,
  ExtensionMcpPreview,
} from "~/stores/relay";
import McpInstallForm from "~/components/extensions/McpInstallForm.vue";
import HookReviewList from "~/components/extensions/HookReviewList.vue";
import { mcpPreviewMatchesExtension } from "~/utils/extensionMcpPreview";
import {
  extensionSupportsClient,
  synchronizeExtensionInstallSelection,
} from "~/utils/extensionInstallSelection";
import { errorText, toRelayError } from "~/utils/errors";

const visible = defineModel<boolean>("visible", { default: false });
const props = defineProps<{
  extension: ExtensionCatalogPackage | null;
  detectedClients: AgentClient[];
}>();
const emit = defineEmits<{
  installed: [kind: ExtensionCatalogPackage["catalogKind"]];
  refreshed: [kind: ExtensionCatalogPackage["catalogKind"]];
}>();
const { invokeLocalCommand } = useLocalCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
const selectedClients = ref<AgentClient[]>([]);
const installing = ref(false);
const mcpPreview = ref<ExtensionMcpPreview | null>(null);
const mcpPreviewLoading = ref(false);
const mcpPreviewError = ref("");
const installResult = ref<ExtensionInstallResult | null>(null);
let mcpPreviewRequest = 0;
const actionLabel = computed(() =>
  props.extension?.installAction === "update"
    ? "更新"
    : props.extension?.installAction === "partial"
      ? "补装"
      : "安装",
);

const detected = computed(() => new Set(props.detectedClients));
const isMcp = computed(() =>
  props.extension?.artifacts.some((artifact) => artifact.kind === "mcp"),
);
const artifactLabels = computed(
  () =>
    props.extension?.artifacts.map((artifact) => artifact.kind.toUpperCase()) ??
    [],
);
const kindIcon = computed(() => {
  const kind = props.extension?.catalogKind;
  if (!kind) return "";
  return kind === "rule" ? "ph:notebook" : "ph:puzzle-piece";
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
const selectableClients = computed(() =>
  supportedDetectedClients.value.filter((client) =>
    extensionSupportsClient(props.extension?.artifacts ?? [], client),
  ),
);
const clientOptions = computed(() => [
  {
    value: "codexCli",
    label: "Codex CLI",
    disabled:
      !detected.value.has("codexCli") ||
      !extensionSupportsClient(props.extension?.artifacts ?? [], "codexCli"),
  },
  {
    value: "chatgpt",
    label: "ChatGPT",
    disabled:
      !detected.value.has("chatgpt") ||
      !extensionSupportsClient(props.extension?.artifacts ?? [], "chatgpt"),
  },
  {
    value: "openCode",
    label: "OpenCode",
    disabled:
      !detected.value.has("openCode") ||
      !extensionSupportsClient(props.extension?.artifacts ?? [], "openCode"),
  },
]);
function selectClients(values: AgentClient[]) {
  if (!props.extension) return;
  selectedClients.value = synchronizeExtensionInstallSelection({
    detected: supportedDetectedClients.value,
    artifacts: props.extension.artifacts,
    next: values,
    previous: selectedClients.value,
  });
}

async function install(overwrite = false) {
  if (!props.extension || !selectedClients.value.length) return;
  if (
    isMcp.value &&
    (!mcpPreview.value ||
      !mcpPreviewMatchesExtension(mcpPreview.value, props.extension))
  ) {
    return;
  }
  installing.value = true;
  try {
    const result = await invokeLocalCommand<ExtensionInstallResult>(
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
    emit("installed", props.extension.catalogKind);
    if (result.review.length) {
      installResult.value = result;
    } else {
      visible.value = false;
    }
  } catch (caught) {
    const error = caught as { code?: string; message?: string };
    if (!overwrite && error.code === "extension_target_exists") {
      const confirmed = await confirmAction({
        title: isMcp.value ? "覆盖已有 MCP 配置" : "覆盖已有技能",
        message: isMcp.value
          ? "目标 MCP 服务名已存在，是否覆盖安装？"
          : "目标技能目录已存在，是否覆盖安装？",
        description: isMcp.value
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
    if (isMcp.value) {
      emit("refreshed", props.extension.catalogKind);
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
      props.extension?.artifacts,
    ] as const,
  async ([isVisible]) => {
    const request = ++mcpPreviewRequest;
    if (!isVisible) return;
    const extension = props.extension;
    installResult.value = null;
    mcpPreview.value = null;
    mcpPreviewError.value = "";
    selectedClients.value = ["partial", "update"].includes(
      extension?.installAction ?? "",
    )
      ? selectableClients.value.filter((client) =>
          extension?.installAction === "partial"
            ? !extension.installedClients.includes(client)
            : extension?.installedClients.includes(client),
        )
      : [...selectableClients.value];
    if (!extension?.artifacts.some((artifact) => artifact.kind === "mcp"))
      return;
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
          <span
            v-if="artifactLabels.length"
            class="extension-install-drawer__artifacts"
          >
            {{ artifactLabels.join(" · ") }}
          </span>
        </div>
        <Tag
          v-if="transportLabel"
          class="extension-install-drawer__identity-transport"
          semantic="info"
          size="small"
        >
          {{ transportLabel }}
        </Tag>
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
      <HookReviewList
        v-if="installResult?.review.length"
        :review="installResult.review"
      />
    </div>
    <template #footer>
      <div class="extension-install-drawer__footer">
        <div class="extension-install-drawer__actions">
          <Button :disabled="installing" @click="visible = false">
            取消
          </Button>
          <template v-if="installResult">
            <Button semantic="primary" variant="solid" @click="visible = false">
              完成
            </Button>
          </template>
          <Button
            v-else
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

.extension-install-drawer__artifacts {
  color: var(--st-text-muted);
  font-size: var(--text-xs);
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
