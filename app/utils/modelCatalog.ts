import { readonly, shallowRef } from "vue";
import type {
  CatalogModelResponse,
  ProviderCatalogResponse,
} from "~/stores/relay";

function emptyCatalog(): ProviderCatalogResponse {
  return {
    language_models: [],
    image_generation_models: [],
    providers: [],
  };
}

const catalog = shallowRef<ProviderCatalogResponse>(emptyCatalog());
const entries = shallowRef(new Map<string, CatalogModelResponse>());

export function setModelCatalog(value?: ProviderCatalogResponse | null) {
  const next = value ?? emptyCatalog();
  catalog.value = next;
  entries.value = new Map(
    [...next.language_models, ...next.image_generation_models].map((model) => [
      model.id,
      model,
    ]),
  );
}

export function modelCatalogEntry(id?: string | null) {
  return id ? entries.value.get(id) : undefined;
}

export function modelCatalogLabel(id?: string | null) {
  if (!id) return "";
  return modelCatalogEntry(id)?.display_name ?? id;
}

export function useModelCatalog() {
  return {
    catalog: readonly(catalog),
    setModelCatalog,
    modelCatalogEntry,
    modelCatalogLabel,
  };
}
