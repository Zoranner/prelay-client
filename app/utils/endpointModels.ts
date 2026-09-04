import type { EndpointModel, Provider } from "~/stores/relay";
import type { CatalogModelResponse } from "~/stores/relay";
import {
  modelCatalogEntry,
  modelCatalogLabel,
  modelCatalogProviderModels,
} from "~/utils/modelCatalog";

export type EndpointModelGroup = {
  name: string;
  displayName: string;
  catalogModel: CatalogModelResponse | undefined;
  mappings: Array<{ model: EndpointModelLike; index: number }>;
};

export type EndpointModelLike = Omit<EndpointModel, "model_name"> &
  Partial<Pick<EndpointModel, "model_name">>;

export function catalogModelsForProvider(provider: Provider) {
  const catalogIds = new Set(
    modelCatalogProviderModels(provider.provider_type).map((model) => model.id),
  );
  return provider.models.filter(
    (model) =>
      catalogIds.has(model.model_name) &&
      Boolean(modelCatalogEntry(model.model_name)),
  );
}

export function groupEndpointModels(
  models: EndpointModelLike[],
): EndpointModelGroup[] {
  const groups = new Map<string, EndpointModelGroup>();
  models.forEach((model, index) => {
    const name = model.model_name?.trim() || model.upstream_model.trim();
    const group = groups.get(name) ?? {
      name,
      displayName: model.display_name?.trim() || modelCatalogLabel(name),
      catalogModel: modelCatalogEntry(name),
      mappings: [],
    };
    group.mappings.push({ model, index });
    groups.set(name, group);
  });
  return [...groups.values()];
}
