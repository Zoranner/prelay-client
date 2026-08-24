import { ref } from "vue";
import { useConfirm, useOverlayManager } from "stellar-ui";

export type WorkspaceExitState = "allow" | "discard" | "blocked";

export interface WorkspaceExitOptions {
  confirmText?: string;
  description?: string;
  message?: string;
  title?: string;
}

interface WorkspaceExitEntry {
  close: () => void;
  state: () => WorkspaceExitState;
}

interface RegisteredWorkspaceExitEntry {
  id: number;
  entry: WorkspaceExitEntry;
}

const entries = ref<RegisteredWorkspaceExitEntry[]>([]);
let nextEntryId = 0;

export function useWorkspaceExitGuard() {
  const { confirm, isOpen: isConfirmOpen } = useConfirm();
  const overlayManager = useOverlayManager();

  function register(entry: WorkspaceExitEntry) {
    const id = ++nextEntryId;
    entries.value = [...entries.value, { id, entry }];

    return {
      unregister() {
        entries.value = entries.value.filter((item) => item.id !== id);
      },
      requestExit(options?: WorkspaceExitOptions) {
        return requestEntryExit(id, options);
      },
    };
  }

  async function requestEntryExit(
    id?: number,
    options: WorkspaceExitOptions = {},
  ) {
    if (isConfirmOpen.value) {
      overlayManager.emphasize();
      return false;
    }

    const entriesToClose = id
      ? entries.value.filter((item) => item.id === id)
      : [...entries.value].reverse();

    for (const current of entriesToClose) {
      const state = current.entry.state();
      if (state === "blocked") {
        overlayManager.emphasize();
        return false;
      }
      if (state === "discard") {
        const confirmed = await confirm({
          title: options.title ?? "放弃未保存的修改？",
          message: options.message ?? "离开后，当前修改不会保存。",
          description: options.description,
          confirmText: options.confirmText ?? "放弃修改",
          danger: true,
        });
        if (!confirmed) return false;
      }

      current.entry.close();
    }

    return true;
  }

  function requestExit(options?: WorkspaceExitOptions) {
    return requestEntryExit(undefined, options);
  }

  return { register, requestExit };
}
