import type {
  ExtensionCatalogKind,
  ExtensionCatalogSnapshot,
} from "~/stores/relay";

function emptyCatalog(): ExtensionCatalogSnapshot {
  return { packages: [] };
}

export const extensionCatalogKinds: ExtensionCatalogKind[] = [
  "rule",
  "plugin",
];

type CatalogState = Record<ExtensionCatalogKind, ExtensionCatalogSnapshot>;
type LoadingState = Record<ExtensionCatalogKind, boolean>;
type LoadedState = Record<ExtensionCatalogKind, boolean>;

function emptyCatalogs(): CatalogState {
  return {
    rule: emptyCatalog(),
    plugin: emptyCatalog(),
  };
}

const loadPromises = new Map<ExtensionCatalogKind, Promise<void>>();
let generation = 0;

export function useExtensionCatalog() {
  const { invokeLocalCommand } = useLocalCommand();
  const catalogs = useState<CatalogState>("extension-catalog", emptyCatalogs);
  const loaded = useState<LoadedState>("extension-catalog-loaded", () => ({
    rule: false,
    plugin: false,
  }));
  const loading = useState<LoadingState>("extension-catalog-loading", () => ({
    rule: false,
    plugin: false,
  }));

  async function load(kind: ExtensionCatalogKind, force = false) {
    if (loaded.value[kind] && !force) return;
    if (loadPromises.has(kind)) return loadPromises.get(kind);

    loading.value[kind] = true;
    const requestGeneration = generation;
    const request = (async () => {
      try {
        const snapshot = await invokeLocalCommand<ExtensionCatalogSnapshot>(
          "extensions_list",
          { kind },
          { notify: false, trackPending: false },
        );
        if (requestGeneration === generation) {
          catalogs.value[kind] = snapshot;
          loaded.value[kind] = true;
        }
      } finally {
        if (requestGeneration === generation) loading.value[kind] = false;
        loadPromises.delete(kind);
      }
    })();
    loadPromises.set(kind, request);
    return request;
  }

  function packages(kind: ExtensionCatalogKind) {
    return computed(() => catalogs.value[kind].packages);
  }

  const updateCounts = computed<Record<ExtensionCatalogKind, number>>(() => ({
    rule: updatePackageCount("rule"),
    plugin: updatePackageCount("plugin"),
  }));
  const totalUpdateCount = computed(() =>
    extensionCatalogKinds.reduce(
      (total, kind) => total + updateCounts.value[kind],
      0,
    ),
  );

  function updatePackageCount(kind: ExtensionCatalogKind) {
    return catalogs.value[kind].packages.filter(
      (extension) => extension.installAction === "update",
    ).length;
  }

  function invalidate() {
    generation += 1;
    loadPromises.clear();
    catalogs.value = emptyCatalogs();
    loaded.value = { rule: false, plugin: false };
    loading.value = { rule: false, plugin: false };
  }

  function applySnapshot(
    kind: ExtensionCatalogKind,
    snapshot: ExtensionCatalogSnapshot,
  ) {
    catalogs.value[kind] = snapshot;
    loaded.value[kind] = true;
  }

  return {
    applySnapshot,
    catalogs,
    loaded,
    loading,
    load,
    packages,
    totalUpdateCount,
    updateCounts,
    invalidate,
  };
}
