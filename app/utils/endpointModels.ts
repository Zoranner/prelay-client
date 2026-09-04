import type { EndpointModel, Provider, ProviderModel } from "~/stores/relay";
import type { CatalogModelResponse } from "~/stores/relay";
import { modelCatalogEntry, modelCatalogLabel } from "~/utils/modelCatalog";

export type EndpointModelGroup = {
  name: string;
  displayName: string;
  catalogModel: CatalogModelResponse | undefined;
  mappings: Array<{ model: EndpointModelLike; index: number }>;
};

export type EndpointModelLike = Omit<EndpointModel, "model_name"> &
  Partial<Pick<EndpointModel, "model_name">>;

export function endpointModelsForProvider(provider: Provider) {
  return provider.models;
}

export function availableEndpointModelsForProvider(
  providerModels: ProviderModel[],
  endpointModels: EndpointModelLike[],
  providerId: string,
  groupName?: string,
) {
  const usedUpstreamModels = new Set(
    endpointModels
      .filter((model) => model.provider_id === providerId)
      .map((model) => model.upstream_model.trim()),
  );
  return providerModels.filter(
    (model) =>
      (!groupName || model.model_name === groupName) &&
      !usedUpstreamModels.has(model.model_name),
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
