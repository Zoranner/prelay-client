import type { ExtensionCatalogSnapshot, ExtensionPackage } from "~/stores/relay";

function emptyCatalog(): ExtensionCatalogSnapshot {
  return { packages: [] };
}

let loadPromise: Promise<void> | undefined;

export function useExtensionCatalog() {
  const { invokeLocalCommand } = useLocalCommand();
  const catalog = useState<ExtensionCatalogSnapshot>(
    "extension-catalog",
    emptyCatalog,
  );
  const loaded = useState("extension-catalog-loaded", () => false);
  const loading = useState("extension-catalog-loading", () => false);

  async function load(force = false) {
    if (loaded.value && !force) return;
    if (loadPromise) return loadPromise;

    loading.value = true;
    loadPromise = (async () => {
      try {
        catalog.value = await invokeLocalCommand<ExtensionCatalogSnapshot>(
          "extensions_list",
          undefined,
          { notify: false, trackPending: false },
        );
        loaded.value = true;
      } finally {
        loading.value = false;
        loadPromise = undefined;
      }
    })();
    return loadPromise;
  }

  function packagesByKind(kind: ExtensionPackage["kind"]) {
    return computed(() =>
      catalog.value.packages.filter((item) => item.kind === kind),
    );
  }

  return { catalog, loaded, loading, load, packagesByKind };
}
