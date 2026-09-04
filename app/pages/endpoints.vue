<script setup lang="ts">
import {
  useRelayStore,
  type BootstrapState,
  type EndpointModel,
  type Provider,
  type RelayEndpoint,
} from "~/stores/relay";
import { Button, Drawer, useConfirm, useNotification } from "@stellar/ui";
import EndpointForm from "~/components/endpoints/EndpointForm.vue";
import EndpointList from "~/components/endpoints/EndpointList.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

type EndpointFormPayload = {
  id?: string;
  name: string;
  protocol: string;
  models: Array<Pick<EndpointModel, "provider_id" | "upstream_model">>;
};

const { pending, invokeCommand } = useRelayCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
const workspaceExit = useWorkspaceExitGuard();
const { bootstrap, setBootstrap } = useRelayStore();
const providers = ref<Provider[]>([]);
const endpoints = ref<RelayEndpoint[]>([]);
const editingEndpoint = ref<RelayEndpoint | null>(null);
const showForm = ref(false);
const formDirty = ref(false);
let exitRegistration: ReturnType<typeof workspaceExit.register> | undefined;

async function load() {
  try {
    const [providerList, endpointList, bootstrapState] = await Promise.all([
      invokeCommand<Provider[]>("providers_list"),
      invokeCommand<RelayEndpoint[]>("endpoints_list"),
      bootstrap.value
        ? Promise.resolve(bootstrap.value)
        : invokeCommand<BootstrapState>("bootstrap"),
    ]);
    providers.value = providerList;
    endpoints.value = endpointList;
    setBootstrap(bootstrapState);
  } catch {
    // The command composable exposes the error to this view.
  }
}

async function saveEndpoint(payload: EndpointFormPayload) {
  try {
    await invokeCommand("endpoints_save", {
      ...(payload.id ? { endpointId: payload.id } : {}),
      input: {
        name: payload.name,
        protocol: payload.protocol,
        models: payload.models.map(({ provider_id, upstream_model }) => ({
          provider_id,
          upstream_model,
        })),
      },
    });
    editingEndpoint.value = null;
    showForm.value = false;
    await load();
    notifications.success("接入点已保存");
  } catch {
    // The command composable exposes the error to this view.
  }
}

async function deleteEndpoint(item: RelayEndpoint) {
  const confirmed = await confirmAction({
    title: "删除接入点",
    message: `删除接入点“${item.name}”？`,
    description: "删除后无法恢复。",
    confirmText: "删除",
    danger: true,
  });
  if (!confirmed) return;
  try {
    await invokeCommand("endpoints_delete", { endpointId: item.id });
    await load();
    notifications.success("接入点已删除");
  } catch {
    // The command composable exposes the error to this view.
  }
}

async function regenerateToken(item: RelayEndpoint) {
  const confirmed = await confirmAction({
    title: "重置 API Token",
    message: `重置“${item.name}”的 Endpoint Token？`,
    description: "现有工具将立即失效。",
    confirmText: "重置",
    danger: true,
  });
  if (!confirmed) return;
  try {
    await invokeCommand("endpoints_regenerate_token", {
      endpointId: item.id,
    });
    await load();
    notifications.success("API Token 已重置");
  } catch {
    // The command composable exposes the error to this view.
  }
}

async function copy(value: string) {
  try {
    await navigator.clipboard.writeText(value);
    notifications.success("已复制到剪贴板");
  } catch {
    notifications.danger("请手动复制。", { title: "无法访问剪贴板" });
  }
}

function edit(item: RelayEndpoint) {
  formDirty.value = false;
  editingEndpoint.value = item;
  showForm.value = true;
}

function createEndpoint() {
  formDirty.value = false;
  editingEndpoint.value = null;
  showForm.value = true;
}

function closeFormImmediately() {
  formDirty.value = false;
  editingEndpoint.value = null;
  showForm.value = false;
}

function requestCloseForm() {
  if (exitRegistration) void exitRegistration.requestExit();
  else closeFormImmediately();
}

function updateFormVisibility(visible: boolean) {
  if (visible) showForm.value = true;
  else requestCloseForm();
}

watch(showForm, (visible) => {
  if (!visible) {
    exitRegistration?.unregister();
    exitRegistration = undefined;
    return;
  }
  exitRegistration = workspaceExit.register({
    close: closeFormImmediately,
    state: () =>
      pending.value ? "blocked" : formDirty.value ? "discard" : "allow",
  });
});

onBeforeUnmount(() => exitRegistration?.unregister());

onMounted(load);
</script>

<template>
  <main class="page-dashboard">
    <PanelSection title="接入点">
      <template #header-actions>
        <Button :disabled="pending" @click="load">
          {{ pending ? "刷新中..." : "刷新" }}
        </Button>
        <Button
          variant="primary"
          icon="ph:plus"
          :disabled="pending"
          @click="createEndpoint"
        >
          新增
        </Button>
      </template>
      <EndpointList
        :endpoints="endpoints"
        :pending="pending"
        @edit="edit"
        @remove="deleteEndpoint"
        @regenerate="regenerateToken"
        @copy="copy"
      />
    </PanelSection>
    <Drawer
      :visible="showForm"
      :title="editingEndpoint ? '编辑接入点' : '新建接入点'"
      size="xlarge"
      :blocked="pending || formDirty"
      @update:visible="updateFormVisibility"
    >
      <EndpointForm
        :endpoint="editingEndpoint"
        :providers="providers"
        :pending="pending"
        @dirty-change="formDirty = $event"
        @save="saveEndpoint"
        @cancel="requestCloseForm"
      />
      <template #footer>
        <Button @click="requestCloseForm">取消</Button>
        <Button
          form="endpoint-form"
          type="submit"
          variant="primary"
          :disabled="pending"
        >
          {{ pending ? "保存中..." : "保存" }}
        </Button>
      </template>
    </Drawer>
  </main>
</template>

<style scoped>
.page-dashboard {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--pr-dashboard-padding);
}
</style>
