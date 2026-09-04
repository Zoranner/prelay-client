import { readonly, ref, shallowRef } from "vue";
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
const status = ref<ModelCatalogStatus>("idle");

export type ModelCatalogStatus = "idle" | "loading" | "ready" | "error";

export type ModelCatalogRequest = () => Promise<ProviderCatalogResponse>;

export function setModelCatalog(
  value?: ProviderCatalogResponse | null,
  nextStatus?: ModelCatalogStatus,
) {
  const next = value ?? emptyCatalog();
  catalog.value = next;
  entries.value = new Map(
    [...next.language_models, ...next.image_generation_models].map((model) => [
      model.id,
      model,
    ]),
  );
  status.value = nextStatus ?? (value ? "ready" : "idle");
}

export function setModelCatalogStatus(next: ModelCatalogStatus) {
  status.value = next;
  if (next !== "ready") setModelCatalog(undefined, next);
}

export async function loadModelCatalogRequest(
  request: ModelCatalogRequest,
  isCurrent: () => boolean,
) {
  setModelCatalogStatus("loading");
  try {
    const value = await request();
    if (isCurrent()) setModelCatalog(value);
  } catch {
    if (isCurrent()) setModelCatalogStatus("error");
  }
}

export function modelCatalogEntry(id?: string | null) {
  return id ? entries.value.get(id) : undefined;
}

export function modelCatalogLabel(id?: string | null) {
  if (!id) return "";
  return modelCatalogEntry(id)?.display_name ?? id;
}

export function modelCatalogProviderModels(providerId: string) {
  const provider = catalog.value.providers.find(
    (candidate) => candidate.id === providerId,
  );
  if (!provider) return [];
  return [...provider.language_models, ...provider.image_generation_models]
    .map((id) => entries.value.get(id))
    .filter((model): model is CatalogModelResponse => Boolean(model));
}

export function useModelCatalog() {
  return {
    catalog: readonly(catalog),
    status: readonly(status),
    setModelCatalog,
    setModelCatalogStatus,
    modelCatalogEntry,
    modelCatalogLabel,
    modelCatalogProviderModels,
  };
}
