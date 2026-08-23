import { invoke } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

import { toRelayError, type RelayError } from "~/utils/errors";

export type LocalCommand = "extensions_list";

export function useLocalCommand() {
  const pendingRequests = ref(0);
  const error = ref<RelayError | null>(null);
  const pending = computed(() => pendingRequests.value > 0);

  async function invokeLocalCommand<T>(command: LocalCommand): Promise<T> {
    pendingRequests.value += 1;
    error.value = null;
    try {
      return await invoke<T>(command);
    } catch (caught) {
      const localError = toRelayError(caught);
      error.value = localError;
      throw localError;
    } finally {
      pendingRequests.value = Math.max(0, pendingRequests.value - 1);
    }
  }

  return { pending, error, invokeLocalCommand };
}
