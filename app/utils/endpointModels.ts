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
    const name = modelGroupName(model);
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

export function modelGroupName(model: EndpointModelLike) {
  return model.model_name?.trim() || model.upstream_model.trim();
}

export function moveEndpointMapping<T extends EndpointModelLike>(
  models: readonly T[],
  index: number,
  delta: number,
): T[] {
  const current = models[index];
  if (!current) {
    return [...models];
  }
  const name = modelGroupName(current);
  const positions = models
    .map((model, position) => (modelGroupName(model) === name ? position : -1))
    .filter((position) => position !== -1);
  const position = positions.indexOf(index);
  const target = position === -1 ? -1 : (positions[position + delta] ?? -1);
  const from = models[index];
  const to = target === -1 ? undefined : models[target];
  if (!from || !to) {
    return [...models];
  }
  const next = [...models];
  next[index] = to;
  next[target] = from;
  return next;
}

export type EndpointMappingCheck = {
  providerId: string;
  upstreamModel: string;
  providerModels: EndpointProviderModelOption[];
  endpointModels: EndpointModelLike[];
  fixedModelName?: string;
};

export type EndpointMappingFailure = {
  message: string;
  title: string;
};

export function checkEndpointMapping(
  check: EndpointMappingCheck,
): EndpointMappingFailure | null {
  if (!check.providerId || !check.upstreamModel) {
    return { message: "请选择供应商和上游模型。", title: "模型配置不完整" };
  }
  if (
    !check.providerModels.some(
      (model) => model.model_name === check.upstreamModel,
    )
  ) {
    return { message: "请选择该供应商已配置的模型。", title: "上游模型无效" };
  }
  if (check.fixedModelName && check.upstreamModel !== check.fixedModelName) {
    return { message: "只能添加相同名称的上游模型。", title: "模型不匹配" };
  }
  if (
    check.endpointModels.some(
      (mapping) =>
        mapping.provider_id === check.providerId &&
        mapping.upstream_model === check.upstreamModel,
    )
  ) {
    return { message: "该供应商已经绑定此模型。", title: "模型已存在" };
  }
  return null;
}
