import type { EndpointModel } from "~/stores/relay";
import type { CatalogModelResponse } from "~/stores/relay";
import { modelCatalogEntry, modelCatalogLabel } from "~/utils/modelCatalog";

export type EndpointModelGroup = {
  name: string;
  displayName: string;
  catalogModel: CatalogModelResponse | undefined;
  mappings: Array<{ model: EndpointModel; index: number }>;
};

export function groupEndpointModels(
  models: EndpointModel[],
): EndpointModelGroup[] {
  const groups = new Map<string, EndpointModelGroup>();
  models.forEach((model, index) => {
    const name = model.model_name.trim() || model.upstream_model.trim();
    const group = groups.get(name) ?? {
      name,
      displayName: modelCatalogLabel(name),
      catalogModel: modelCatalogEntry(name),
      mappings: [],
    };
    group.mappings.push({ model, index });
    groups.set(name, group);
  });
  return [...groups.values()];
}
