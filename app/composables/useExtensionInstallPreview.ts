import type {
  ExtensionCatalogPackage,
  ExtensionMcpPreview,
} from "~/stores/relay";
import {
  hookPreviewMatchesExtension,
  type HookInstallPreview,
} from "~/utils/extensionHookPreview";
import { mcpPreviewMatchesExtension } from "~/utils/extensionMcpPreview";
import { errorText, toRelayError } from "~/utils/errors";

export function useExtensionInstallPreview() {
  const { invokeLocalCommand } = useLocalCommand();
  const mcpPreview = ref<ExtensionMcpPreview | null>(null);
  const mcpLoading = ref(false);
  const mcpError = ref("");
  const hookPreview = ref<HookInstallPreview | null>(null);
  const hookLoading = ref(false);
  const hookError = ref("");
  let request = 0;

  function reset() {
    mcpPreview.value = null;
    mcpLoading.value = false;
    mcpError.value = "";
    hookPreview.value = null;
    hookLoading.value = false;
    hookError.value = "";
  }

  async function load(extension: ExtensionCatalogPackage | null) {
    const current = ++request;
    reset();
    if (!extension) return;
    if (extension.artifacts.some((artifact) => artifact.kind === "mcp")) {
      mcpLoading.value = true;
      try {
        const preview = await invokeLocalCommand<ExtensionMcpPreview>(
          "extensions_mcp_preview",
          { package: extension },
          { notify: false, trackPending: false },
        );
        if (
          current === request &&
          mcpPreviewMatchesExtension(preview, extension)
        ) {
          mcpPreview.value = preview;
        }
      } catch (caught) {
        if (current === request) {
          mcpError.value = errorText(toRelayError(caught));
        }
      } finally {
        if (current === request) mcpLoading.value = false;
      }
    }
    if (current !== request) return;
    if (extension.artifacts.some((artifact) => artifact.kind === "hook")) {
      hookLoading.value = true;
      try {
        const preview = await invokeLocalCommand<HookInstallPreview>(
          "extensions_hook_preview",
          { package: extension },
          { notify: false, trackPending: false },
        );
        if (
          current === request &&
          hookPreviewMatchesExtension(preview, extension)
        ) {
          hookPreview.value = preview;
        }
      } catch (caught) {
        if (current === request) {
          hookError.value = errorText(toRelayError(caught));
        }
      } finally {
        if (current === request) hookLoading.value = false;
      }
    }
  }

  return {
    mcpPreview,
    mcpLoading,
    mcpError,
    hookPreview,
    hookLoading,
    hookError,
    reset,
    load,
  };
}
