<script setup lang="ts">
import type {
  CatalogProvider,
  IdentityDirectoryEntry,
  Provider,
  ProviderListItem,
  ProviderSharing,
  ProviderSharingInput,
  ProviderUsage,
  UpstreamProtocol,
} from "~/stores/relay";
import type { ProviderFormPayload } from "~/composables/useProviderForm";
import { type ProviderOperationResult } from "~/utils/providerOperations";
import { Button, Drawer, useConfirm, useNotification } from "@stellar/ui";
import ProviderForm from "~/components/providers/ProviderForm.vue";
import ProviderList from "~/components/providers/ProviderList.vue";
import ProviderSharingDrawer from "~/components/providers/ProviderSharingDrawer.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

const { pending, invokeCommand } = useRelayCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
const workspaceExit = useWorkspaceExitGuard();
const providers = ref<ProviderListItem[]>([]);
const catalogProviders = ref<CatalogProvider[]>([]);
const editingProvider = ref<EditableProvider | null>(null);
const showForm = ref(false);
const loadingProviders = ref(false);
const pingStates = ref<Record<string, ProviderPingState>>({});
const sharingProvider = ref<ProviderListItem | null>(null);
const sharing = ref<ProviderSharing | null>(null);
const identities = ref<IdentityDirectoryEntry[]>([]);
const sharingUsage = ref<ProviderUsage | null>(null);
const showSharing = ref(false);
const loadingSharing = ref(false);
const formDirty = ref(false);
let exitRegistration: ReturnType<typeof workspaceExit.register> | undefined;

type ProviderPingState = {
  checking: boolean;
  ok?: boolean;
  latencyMs?: number | null;
};

type EditableProvider = ProviderListItem & {
  api_key?: string;
  api_key_masked?: string;
};

async function loadProviders() {
  loadingProviders.value = true;
  try {
    const [availableProviders, availableCatalogProviders, directory] =
      await Promise.all([
        invokeCommand<ProviderListItem[]>("providers_list"),
        invokeCommand<CatalogProvider[]>("catalog_providers_list"),
        invokeCommand<IdentityDirectoryEntry[]>(
          "identity_directory_list",
        ).catch(() => []),
      ]);
    providers.value = availableProviders;
    catalogProviders.value = availableCatalogProviders;
    identities.value = directory;
    pingStates.value = Object.fromEntries(
      providers.value.map((provider) => [provider.id, { checking: false }]),
    );
    void Promise.all(providers.value.map(pingProvider));
  } catch {
    // The command composable exposes the error to this view.
  } finally {
    loadingProviders.value = false;
  }
}

function emptyUsage(): ProviderUsage {
  return {
    total_requests: 0,
    input_tokens: 0,
    output_tokens: 0,
    total_tokens: 0,
    latest_used_at: null,
    users: [],
  };
}

async function pingProvider(provider: ProviderListItem) {
  pingStates.value = {
    ...pingStates.value,
    [provider.id]: { checking: true },
  };
  try {
    const result = await invokeCommand<ProviderOperationResult>(
      "providers_ping",
      {
        providerId: provider.id,
      },
    );
    pingStates.value = {
      ...pingStates.value,
      [provider.id]: {
        checking: false,
        ok: result.ok,
        latencyMs: result.latency_ms,
      },
    };
  } catch {
    pingStates.value = {
      ...pingStates.value,
      [provider.id]: { checking: false, ok: false },
    };
  }
}

async function saveProvider(payload: ProviderFormPayload) {
  try {
    await invokeCommand("providers_save", {
      ...(payload.id ? { providerId: payload.id } : {}),
      input: {
        name: payload.name,
        provider_type: payload.provider_type,
        base_url: payload.base_url,
        api_key: payload.api_key,
        capabilities: payload.capabilities,
      },
    });
    showForm.value = false;
    editingProvider.value = null;
    await loadProviders();
    notifications.success("供应商已保存");
  } catch {
    // The command composable exposes the error to this view.
  } finally {
    payload.api_key = "";
  }
}

async function deleteProvider(provider: ProviderListItem) {
  const confirmed = await confirmAction({
    title: "删除供应商",
    message: `删除供应商“${provider.name}”？`,
    description: "该供应商及其模型将一并删除，且无法恢复。",
    confirmText: "删除",
    danger: true,
  });
  if (!confirmed) return;
  try {
    await invokeCommand("providers_delete", { providerId: provider.id });
    await loadProviders();
    notifications.success("供应商已删除");
  } catch {
    // The command composable exposes the error to this view.
  }
}

function testProtocolFromForm(input: {
  provider_type: string;
  base_url: string;
  api_key: string;
  protocol?: UpstreamProtocol;
  model?: string;
}) {
  return invokeCommand<ProviderOperationResult>("providers_test_protocol", {
    input,
  });
}

async function editProvider(provider: ProviderListItem) {
  formDirty.value = false;
  let editable: EditableProvider = provider;
  try {
    const detail = await invokeCommand<Provider>("providers_get", {
      providerId: provider.id,
    });
    editable = { ...provider, ...detail };
  } catch {
    // The command composable exposes the error to this view.
  }
  editingProvider.value = editable;
  showForm.value = true;
}

async function openProviderSharing(provider: ProviderListItem) {
  sharingProvider.value = provider;
  sharing.value = null;
  sharingUsage.value = null;
  showSharing.value = true;
  loadingSharing.value = true;
  try {
    const [sharingResponse, identityList, usageResponse] = await Promise.all([
      invokeCommand<ProviderSharing>("providers_sharing_get", {
        providerId: provider.id,
      }),
      invokeCommand<IdentityDirectoryEntry[]>("identity_directory_list"),
      invokeCommand<ProviderUsage>("providers_usage_get", {
        providerId: provider.id,
        range: "all",
      }),
    ]);
    sharing.value = sharingResponse;
    identities.value = identityList;
    sharingUsage.value = usageResponse;
  } catch {
    // The command composable exposes the error to this view.
  } finally {
    loadingSharing.value = false;
  }
}

function closeProviderSharing() {
  showSharing.value = false;
  sharingProvider.value = null;
  sharing.value = null;
  sharingUsage.value = null;
}

async function saveSharing(input: ProviderSharingInput) {
  if (!sharingProvider.value) return;
  try {
    sharing.value = await invokeCommand<ProviderSharing>(
      "providers_sharing_save",
      {
        providerId: sharingProvider.value.id,
        input,
      },
    );
    await loadProviders();
    const refreshedProvider = providers.value.find(
      (provider) => provider.id === sharingProvider.value?.id,
    );
    if (refreshedProvider) {
      sharingProvider.value = refreshedProvider;
      try {
        sharingUsage.value = await invokeCommand<ProviderUsage>(
          "providers_usage_get",
          {
            providerId: refreshedProvider.id,
            range: "all",
          },
        );
      } catch {
        sharingUsage.value = emptyUsage();
      }
    } else {
      closeProviderSharing();
    }
    notifications.success("共享设置已保存");
  } catch {
    // The command composable exposes the error to this view.
  }
}

function newProvider() {
  formDirty.value = false;
  editingProvider.value = null;
  showForm.value = true;
}

function closeFormImmediately() {
  formDirty.value = false;
  showForm.value = false;
  editingProvider.value = null;
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

onMounted(loadProviders);
</script>

<template>
  <main class="page-dashboard">
    <PanelSection title="供应商">
      <template #header-actions>
        <Button :disabled="loadingProviders" @click="loadProviders">
          {{ loadingProviders ? "刷新中..." : "刷新" }}
        </Button>
        <Button
          semantic="primary"
          variant="solid"
          icon="ph:plus"
          @click="newProvider"
          >新增</Button
        >
      </template>
      <ProviderList
        :loading="loadingProviders"
        :providers="providers"
        :identities="identities"
        :ping-states="pingStates"
        @edit="editProvider"
        @ping="pingProvider"
        @remove="deleteProvider"
        @share="openProviderSharing"
      />
    </PanelSection>
    <Drawer
      :visible="showForm"
      :title="editingProvider ? '编辑供应商' : '新增供应商'"
      size="xlarge"
      :blocked="pending || formDirty"
      @update:visible="updateFormVisibility"
    >
      <ProviderForm
        :provider="editingProvider"
        :catalog-providers="catalogProviders"
        :pending="pending"
        :can-edit="editingProvider ? editingProvider.can_manage : true"
        :can-test="editingProvider ? editingProvider.can_manage : true"
        :test-protocol="testProtocolFromForm"
        @save="saveProvider"
        @dirty-change="formDirty = $event"
        @cancel="requestCloseForm"
      />
      <template #footer>
        <Button @click="requestCloseForm">取消</Button>
        <Button
          form="provider-form"
          type="submit"
          semantic="primary"
          variant="solid"
          :disabled="pending"
        >
          {{ pending ? "保存中..." : "保存" }}
        </Button>
      </template>
    </Drawer>
    <ProviderSharingDrawer
      v-model:visible="showSharing"
      :provider="sharingProvider"
      :sharing="sharing"
      :identities="identities"
      :usage="sharingUsage"
      :pending="pending"
      :loading="loadingSharing"
      @save-sharing="saveSharing"
      @close="closeProviderSharing"
    />
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
