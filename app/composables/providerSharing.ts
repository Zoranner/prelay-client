import type { IdentityDirectoryEntry, ProviderListItem } from "~/types/relay";
import type { ProviderSharing, ProviderSharingInput } from "~/types/quota";
import { useNotification } from "@stellar/ui";

type ProviderSharingOptions = {
  providers: Ref<ProviderListItem[]>;
  identities: Ref<IdentityDirectoryEntry[]>;
  loadProviders: () => Promise<void>;
};

export function useProviderSharing(options: ProviderSharingOptions) {
  const { invokeCommand } = useRelayCommand();
  const notifications = useNotification();
  const provider = ref<ProviderListItem | null>(null);
  const sharing = ref<ProviderSharing | null>(null);
  const visible = ref(false);
  const loading = ref(false);

  async function open(next: ProviderListItem) {
    provider.value = next;
    sharing.value = null;
    visible.value = true;
    loading.value = true;
    try {
      const [sharingResponse, identityList] = await Promise.all([
        invokeCommand<ProviderSharing>("providers_sharing_get", {
          providerId: next.id,
        }),
        invokeCommand<IdentityDirectoryEntry[]>("identity_directory_list"),
      ]);
      sharing.value = sharingResponse;
      options.identities.value = identityList;
    } catch {
      // The command composable exposes the error to this view.
    } finally {
      loading.value = false;
    }
  }

  function close() {
    visible.value = false;
    provider.value = null;
    sharing.value = null;
  }

  async function save(input: ProviderSharingInput) {
    if (!provider.value) return;
    try {
      sharing.value = await invokeCommand<ProviderSharing>(
        "providers_sharing_save",
        {
          providerId: provider.value.id,
          input,
        },
      );
      await options.loadProviders();
      const refreshedProvider = options.providers.value.find(
        (candidate) => candidate.id === provider.value?.id,
      );
      if (refreshedProvider) {
        provider.value = refreshedProvider;
      } else {
        close();
      }
      notifications.success("共享设置已保存");
    } catch {
      // The command composable exposes the error to this view.
    }
  }

  return {
    close,
    loading,
    open,
    provider,
    save,
    sharing,
    visible,
  };
}
