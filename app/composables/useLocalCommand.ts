import { invoke } from "@tauri-apps/api/core";
import { computed, ref } from "vue";
import { useNotification } from "@stellar/ui";

import { toRelayError, type RelayError } from "~/utils/errors";

export type LocalCommand =
  | "agents_list"
  | "agents_versions"
  | "agents_remove"
  | "agent_settings_get"
  | "agent_settings_save"
  | "extensions_list"
  | "extension_readme"
  | "extensions_install";

export function useLocalCommand() {
  const notifications = useNotification();
  const pendingRequests = ref(0);
  const error = ref<RelayError | null>(null);
  const pending = computed(() => pendingRequests.value > 0);

  async function invokeLocalCommand<T>(
    command: LocalCommand,
    payload?: Record<string, unknown>,
    options: { notify?: boolean; trackPending?: boolean } = {},
  ): Promise<T> {
    const notify = options.notify ?? true;
    const trackPending = options.trackPending ?? true;
    if (trackPending) pendingRequests.value += 1;
    if (notify) error.value = null;
    try {
      return await invoke<T>(command, payload);
    } catch (caught) {
      const localError = toRelayError(caught);
      if (notify) {
        error.value = localError;
        notifications.danger(localError.message, { title: "本地操作失败" });
      }
      throw localError;
    } finally {
      if (trackPending) {
        pendingRequests.value = Math.max(0, pendingRequests.value - 1);
      }
    }
  }

  return { pending, error, invokeLocalCommand };
}
