import type {
  ExtensionCatalogKind,
  ExtensionCatalogPackage,
} from "~/types/relay";

type ExtensionLibraryOptions = {
  relayUrl: Readonly<Ref<string | null | undefined>>;
};

export function useExtensionLibrary(options: ExtensionLibraryOptions) {
  const catalog = useExtensionCatalog();
  const actions = useExtensionUpdates();
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

  watch(activeSection, (kind) => {
    void catalog.load(kind, true);
  });

  watch(options.relayUrl, () => {
    catalog.invalidate();
    void Promise.all(extensionCatalogKinds.map((kind) => catalog.load(kind)));
  });

  onMounted(() => {
    void Promise.all(extensionCatalogKinds.map((kind) => catalog.load(kind)));
  });

  return {
    activeSection,
    loading,
    openDetails,
    openInstall,
    packages,
    refresh: actions.refresh,
    selected,
    showDetails,
    showInstall,
    updates,
  };
}
