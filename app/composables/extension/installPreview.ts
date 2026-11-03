import type {
  ExtensionCatalogPackage,
  ExtensionMcpPreview,
} from "~/types/relay";
import { mcpPreviewMatchesExtension } from "~/utils/extension/mcpPreview";
import { errorText, toRelayError } from "~/utils/errors";

export function useExtensionInstallPreview() {
  const { invokeLocalCommand } = useLocalCommand();
  const mcpPreview = ref<ExtensionMcpPreview | null>(null);
  const mcpLoading = ref(false);
  const mcpError = ref("");
  let request = 0;

  function reset() {
    mcpPreview.value = null;
    mcpLoading.value = false;
    mcpError.value = "";
  }

  async function load(extension: ExtensionCatalogPackage | null) {
    const current = ++request;
    reset();
    if (!extension) return;
    if (!extension.artifacts.some((artifact) => artifact.kind === "mcp")) {
      return;
    }

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

  return {
    mcpPreview,
    mcpLoading,
    mcpError,
    reset,
    load,
  };
}
