import type {
  ExtensionCatalogKind,
  ExtensionCatalogPackage,
} from "~/types/relay";

type ExtensionLibraryOptions = {
  refreshClient: () => Promise<void>;
  relayUrl: Readonly<Ref<string | null | undefined>>;
  visible: Readonly<Ref<boolean>>;
};

export function useExtensionLibrary(options: ExtensionLibraryOptions) {
  const catalog = useExtensionCatalog();
  const actions = useExtensionUpdates({
    refreshClient: options.refreshClient,
  });
  const activeSection = ref<ExtensionCatalogKind>("rule");
  const selected = ref<ExtensionCatalogPackage | null>(null);
  const showDetails = ref(false);
  const showInstall = ref(false);
  const packages = computed(
    () => catalog.catalogs.value[activeSection.value].packages,
  );
  const loading = computed(() => catalog.loading.value[activeSection.value]);
  const updates = computed(
    () => catalog.updateCounts.value[activeSection.value],
  );

  function openDetails(extension: ExtensionCatalogPackage) {
    selected.value = extension;
    showDetails.value = true;
  }

  function openInstall(extension: ExtensionCatalogPackage) {
    selected.value = extension;
    showInstall.value = true;
  }

  function updateAll() {
    return actions.updateAll(activeSection.value);
  }

  watch(activeSection, (kind) => {
    if (options.visible.value) void catalog.load(kind, true);
  });

  watch(options.visible, (visible) => {
    if (visible) void catalog.load(activeSection.value);
  });

  watch(options.relayUrl, () => {
    catalog.invalidate();
    void Promise.all(extensionCatalogKinds.map((kind) => catalog.load(kind)));
    if (options.visible.value) void catalog.load(activeSection.value);
  });

  onMounted(() => {
    void Promise.all(extensionCatalogKinds.map((kind) => catalog.load(kind)));
  });

  return {
    activeSection,
    installed: actions.installed,
    loading,
    openDetails,
    openInstall,
    packages,
    refresh: actions.refresh,
    selected,
    showDetails,
    showInstall,
    totalUpdates: catalog.totalUpdateCount,
    updateAll,
    updates,
    updating: actions.updating,
  };
}
