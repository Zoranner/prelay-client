import type { EndpointModel, Provider, ProviderListItem } from "~/stores/relay";
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

export type EndpointProviderModelOption = {
  model_name: string;
  display_name: string;
};

export type EndpointProviderOption = {
  label: string;
  value: string;
  read_only: boolean;
};

export function providerOptionLabel(
  provider: Pick<Provider, "id" | "name"> &
    Partial<Pick<ProviderListItem, "owner_display_name" | "can_manage">>,
) {
  const name = provider.name || provider.id;
  const readOnly = provider.can_manage === false;
  return {
    label: !readOnly
      ? name
      : `${name}（共享自 ${provider.owner_display_name || "其他身份"}）`,
    value: provider.id,
    read_only: readOnly,
  } satisfies EndpointProviderOption;
}

export function endpointModelsForProvider(provider: Provider) {
  return modelCatalogProviderModels(provider.provider_type).map((model) => ({
    model_name: model.id,
    display_name: model.display_name,
  }));
}

export function availableEndpointModelsForProvider(
  providerModels: EndpointProviderModelOption[],
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
