<script setup lang="ts">
import type {
  Provider,
  ProviderCapabilities,
  UpstreamProtocol,
} from "~/stores/relay";
import { type ProviderOperationResult } from "~/utils/providerOperations";
import { Button, Drawer, useConfirm, useNotification } from "stellar-ui";
import ProviderForm from "~/components/providers/ProviderForm.vue";
import ProviderList from "~/components/providers/ProviderList.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

type ProviderFormPayload = {
  id?: string;
  name: string;
  provider_type: string;
  base_url: string;
  api_key: string;
  capabilities: ProviderCapabilities;
  models: string[];
};

const { pending, invokeCommand } = useRelayCommand();
const { confirm: confirmAction } = useConfirm();
const notifications = useNotification();
const providers = ref<Provider[]>([]);
const editingProvider = ref<Provider | null>(null);
const showForm = ref(false);
const loadingProviders = ref(false);
const pingStates = ref<Record<string, ProviderPingState>>({});

type ProviderPingState = {
  checking: boolean;
  ok?: boolean;
  latencyMs?: number | null;
};

async function loadProviders() {
  loadingProviders.value = true;
  try {
    providers.value = await invokeCommand<Provider[]>("providers_list");
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

async function pingProvider(provider: Provider) {
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
        models: payload.models,
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

async function deleteProvider(provider: Provider) {
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

async function discoverModelsFromForm(input: {
  provider_type: string;
  base_url: string;
  api_key: string;
}) {
  return invokeCommand<ProviderOperationResult>("providers_discover_models", {
    input,
  });
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

function editProvider(provider: Provider) {
  editingProvider.value = provider;
  showForm.value = true;
}

function newProvider() {
  editingProvider.value = null;
  showForm.value = true;
}

onMounted(loadProviders);
</script>

<template>
  <main class="page-workbench">
    <PanelSection title="供应商">
      <template #header-actions>
        <Button :disabled="loadingProviders" @click="loadProviders">
          {{ loadingProviders ? "刷新中..." : "刷新" }}
        </Button>
        <Button variant="primary" @click="newProvider">新增</Button>
      </template>
      <ProviderList
        :loading="loadingProviders"
        :providers="providers"
        :ping-states="pingStates"
        @edit="editProvider"
        @ping="pingProvider"
        @remove="deleteProvider"
      />
    </PanelSection>
    <Drawer
      v-model:visible="showForm"
      :title="editingProvider ? '编辑供应商' : '新增供应商'"
      size="xlarge"
      @cancel="showForm = false"
    >
      <ProviderForm
        :provider="editingProvider"
        :pending="pending"
        :discover-models="discoverModelsFromForm"
        :test-protocol="testProtocolFromForm"
        @save="saveProvider"
        @cancel="showForm = false"
      />
      <template #footer>
        <Button @click="showForm = false">取消</Button>
        <Button
          form="provider-form"
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
.page-workbench {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--pr-workbench-padding);
}
</style>
