import type { EndpointModel } from "~/stores/relay";

export type EndpointModelGroup = {
  name: string;
  mappings: Array<{ model: EndpointModel; index: number }>;
};

export function groupEndpointModels(
  models: EndpointModel[],
): EndpointModelGroup[] {
  const groups = new Map<string, EndpointModelGroup>();
  models.forEach((model, index) => {
    const name = model.model_name.trim() || model.upstream_model.trim();
    const group = groups.get(name) ?? { name, mappings: [] };
    group.mappings.push({ model, index });
    groups.set(name, group);
  });
  return [...groups.values()];
}
