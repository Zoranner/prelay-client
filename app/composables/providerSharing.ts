import type {
  IdentityDirectoryEntry,
  ProviderListItem,
  ProviderSharing,
  ProviderSharingInput,
  ProviderUsage,
} from "~/types/relay";
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
  const usage = ref<ProviderUsage | null>(null);
  const visible = ref(false);
  const loading = ref(false);

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

  async function open(next: ProviderListItem) {
    provider.value = next;
    sharing.value = null;
    usage.value = null;
    visible.value = true;
    loading.value = true;
    try {
      const [sharingResponse, identityList, usageResponse] = await Promise.all([
        invokeCommand<ProviderSharing>("providers_sharing_get", {
          providerId: next.id,
        }),
        invokeCommand<IdentityDirectoryEntry[]>("identity_directory_list"),
        invokeCommand<ProviderUsage>("providers_usage_get", {
          providerId: next.id,
          range: "all",
        }),
      ]);
      sharing.value = sharingResponse;
      options.identities.value = identityList;
      usage.value = usageResponse;
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
    usage.value = null;
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
        try {
          usage.value = await invokeCommand<ProviderUsage>(
            "providers_usage_get",
            {
              providerId: refreshedProvider.id,
              range: "all",
            },
          );
        } catch {
          usage.value = emptyUsage();
        }
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
    usage,
    visible,
  };
}
