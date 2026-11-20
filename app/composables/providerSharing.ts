import type { IdentityDirectoryEntry, Provider } from "~/types/relay";
import type { ProviderSharing, ProviderSharingInput } from "~/types/quota";
import { useNotification } from "@stellar/ui";

type ProviderSharingOptions = {
  providers: Ref<Provider[]>;
  identities: Ref<IdentityDirectoryEntry[]>;
  loadProviders: () => Promise<void>;
};

export function useProviderSharing(options: ProviderSharingOptions) {
  const { invokeCommand } = useRelayCommand();
  const notifications = useNotification();
  const provider = ref<Provider | null>(null);
  const sharing = ref<ProviderSharing | null>(null);
  const visible = ref(false);
  const loading = ref(false);
  const saving = ref(false);
  const failed = ref(false);

  async function load() {
    if (!provider.value) return;
    loading.value = true;
    failed.value = false;
    try {
      const [sharingResponse, identityList] = await Promise.all([
        invokeCommand<ProviderSharing>("providers_sharing_get", {
          providerId: provider.value.id,
        }),
        invokeCommand<IdentityDirectoryEntry[]>("identity_directory_list"),
      ]);
      sharing.value = sharingResponse;
      options.identities.value = identityList;
    } catch {
      failed.value = true;
    } finally {
      loading.value = false;
    }
  }

  async function open(next: Provider) {
    provider.value = next;
    sharing.value = null;
    failed.value = false;
    visible.value = true;
    await load();
  }

  function close() {
    visible.value = false;
    provider.value = null;
    sharing.value = null;
    failed.value = false;
  }

  async function save(input: ProviderSharingInput) {
    if (!provider.value || saving.value) return;
    saving.value = true;
    try {
      await invokeCommand<ProviderSharing>("providers_sharing_save", {
        providerId: provider.value.id,
        input,
      });
      close();
      await options.loadProviders();
      notifications.success("共享设置已保存");
    } catch {
      // The command composable exposes the error to this view.
    } finally {
      saving.value = false;
    }
  }

  return {
    close,
    failed,
    loading,
    open,
    provider,
    save,
    saving,
    sharing,
    visible,
  };
}
