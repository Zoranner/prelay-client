<script setup lang="ts">
import {
  Button,
  Checkbox,
  Loading,
  Modal,
  Select,
  useConfirm,
  useNotification,
} from "@stellar/ui";
import type {
  AgentClient,
  ExtensionCatalogPackage,
  ExtensionMcpPreview,
} from "~/stores/relay";
import { mcpPreviewMatchesExtension } from "~/utils/extensionMcpPreview";
import { synchronizeExtensionInstallSelection } from "~/utils/extensionInstallSelection";

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
const mcpRiskAccepted = ref(false);
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
const supportedDetectedClients = computed(() =>
  isMcp.value
    ? props.detectedClients.filter((client) => client !== "chatgpt")
    : props.detectedClients,
);
const clientOptions = computed(() =>
  [
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
    {
      value: "claudeCode",
      label: "Claude Code",
      disabled: !detected.value.has("claudeCode"),
    },
  ].filter((client) => !isMcp.value || client.value !== "chatgpt"),
);
const mcpEnvironmentNames = computed(() =>
  mcpPreview.value?.manifest.transport.type === "stdio"
    ? Object.keys(mcpPreview.value.manifest.transport.environment)
    : [],
);
const mcpHeaders = computed(() =>
  mcpPreview.value?.manifest.transport.type === "http"
    ? Object.entries(mcpPreview.value.manifest.transport.headers)
    : [],
);

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
      !mcpPreviewMatchesExtension(mcpPreview.value, props.extension) ||
      !mcpRiskAccepted.value)
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
    notifications.error(error.message ?? "扩展安装失败。", {
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
    mcpRiskAccepted.value = false;
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
      const error = caught as { message?: string };
      if (request === mcpPreviewRequest) {
        mcpPreviewError.value = error.message ?? "无法读取 MCP 配置。";
      }
    } finally {
      if (request === mcpPreviewRequest) mcpPreviewLoading.value = false;
    }
  },
);
</script>

<template>
  <Modal
    :visible="visible"
    :title="extension ? `${actionLabel} ${extension.name}` : '安装扩展'"
    size="large"
    :blocked="installing"
    :show-cancel="false"
    :show-confirm="false"
    @update:visible="(nextVisible) => (visible = nextVisible)"
  >
    <div class="extension-install">
      <section v-if="isMcp" class="mcp-install-preview">
        <Loading v-if="mcpPreviewLoading" visible text="正在读取 MCP 配置..." />
        <template v-else-if="mcpPreview">
          <h3>MCP 配置</h3>
          <dl>
            <dt>服务名</dt>
            <dd>
              <code>{{ mcpPreview.manifest.name }}</code>
            </dd>
            <template v-if="mcpPreview.manifest.transport.type === 'stdio'">
              <dt>传输方式</dt>
              <dd>本地进程</dd>
              <dt>命令</dt>
              <dd>
                <code>{{ mcpPreview.manifest.transport.command[0] }}</code>
              </dd>
              <template v-if="mcpPreview.manifest.transport.command.length > 1">
                <dt>参数</dt>
                <dd>
                  <code>{{
                    mcpPreview.manifest.transport.command.slice(1).join(" ")
                  }}</code>
                </dd>
              </template>
              <dt>工作目录</dt>
              <dd>默认</dd>
            </template>
            <template v-else>
              <dt>传输方式</dt>
              <dd>HTTP</dd>
              <dt>地址</dt>
              <dd>
                <code>{{ mcpPreview.manifest.transport.url }}</code>
              </dd>
            </template>
            <template v-if="mcpEnvironmentNames.length">
              <dt>MCP 环境变量</dt>
              <dd>{{ mcpEnvironmentNames.join("、") }}</dd>
            </template>
            <template v-if="mcpHeaders.length">
              <dt>请求头</dt>
              <dd class="mcp-install-headers">
                <span v-for="[header, variable] in mcpHeaders" :key="header">
                  {{ header }} -> {{ variable }}
                </span>
              </dd>
            </template>
            <dt>启用状态</dt>
            <dd>
              {{ mcpPreview.manifest.transport.enabled ? "启用" : "停用" }}
            </dd>
            <dt>超时时间</dt>
            <dd>
              {{
                mcpPreview.manifest.transport.timeoutMs
                  ? `${mcpPreview.manifest.transport.timeoutMs} ms`
                  : "未设置"
              }}
            </dd>
          </dl>
          <p class="mcp-install-warning">
            首次使用时可能下载依赖或启动本地进程。
          </p>
          <Checkbox
            v-model="mcpRiskAccepted"
            label="我已了解以上 MCP 配置及其运行风险"
          />
        </template>
        <p v-else class="mcp-install-error">{{ mcpPreviewError }}</p>
      </section>
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
        semantic="primary"
        variant="solid"
        :disabled="
          installing ||
          !selectedClients.length ||
          (isMcp && (!mcpPreview || !mcpRiskAccepted))
        "
        @click="install()"
      >
        {{ installing ? `${actionLabel}中...` : actionLabel }}
      </Button>
    </template>
  </Modal>
</template>

<style scoped>
.extension-install {
  position: relative;
}

.mcp-install-preview {
  margin-bottom: var(--spacing-lg);
}

.mcp-install-preview h3 {
  margin: 0 0 var(--spacing-sm);
  font-size: var(--text-sm);
  font-weight: 600;
}

.mcp-install-preview dl {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: var(--spacing-xs) var(--spacing-md);
  margin: 0;
  font-size: var(--text-sm);
}

.mcp-install-preview dt {
  color: var(--st-text-secondary);
}

.mcp-install-preview dd {
  min-width: 0;
  margin: 0;
  overflow-wrap: anywhere;
}

.mcp-install-headers {
  display: grid;
  gap: var(--spacing-xs);
}

.mcp-install-warning,
.mcp-install-error {
  margin: var(--spacing-md) 0;
  font-size: var(--text-sm);
}

.mcp-install-warning {
  color: var(--st-warning);
}

.mcp-install-error {
  color: var(--st-semantic-error);
}
</style>
