import type { EndpointModel, Provider, ProviderListItem } from "~/stores/relay";
import type { CatalogModelResponse } from "~/stores/relay";
import { modelCatalogEntry, modelCatalogLabel } from "~/utils/modelCatalog";

// 接入点模型按目录模型 id 归组，供应商上游名由服务端按目录映射解析后落在 upstream_model。
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
  return (provider.models ?? []).map((modelId) => ({
    model_name: modelId,
    display_name: modelCatalogLabel(modelId),
  }));
}

export function availableEndpointModelsForProvider(
  providerModels: EndpointProviderModelOption[],
  endpointModels: EndpointModelLike[],
  providerId: string,
  groupName?: string,
) {
  const usedModelNames = new Set(
    endpointModels
      .filter((model) => model.provider_id === providerId)
      .map((model) => modelGroupName(model)),
  );
  return providerModels.filter(
    (model) =>
      (!groupName || model.model_name === groupName) &&
      !usedModelNames.has(model.model_name),
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

// 保存接口的 upstream_model 承载模型标识，服务端再换算上游名；
// 编辑回程里的上游名要还原成目录模型 id，否则再保存会被清单校验拒绝。
export function normalizeEndpointModelIdentities<T extends EndpointModelLike>(
  models: readonly T[],
  providerType: (providerId: string) => string | undefined,
  catalogModelsForProvider: (providerType: string) => Array<{ id: string }>,
): T[] {
  return models.map((model) => {
    const providerTypeId = providerType(model.provider_id);
    if (!providerTypeId) return { ...model };
    const catalogModels = catalogModelsForProvider(providerTypeId);
    const storedIdentity = model.model_name?.trim();
    const catalogModel =
      catalogModels.find((candidate) => candidate.id === storedIdentity) ??
      catalogModels.find(
        (candidate) => candidate.id === model.upstream_model.trim(),
      );
    return catalogModel
      ? { ...model, upstream_model: catalogModel.id }
      : { ...model };
  });
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

export type EndpointMappingIssue = "provider_unavailable" | "model_removed";

export type EndpointMappingProvider = Pick<
  ProviderListItem,
  "id" | "name" | "models"
>;

export type EndpointMappingProblem = {
  model: EndpointModelLike;
  issue: EndpointMappingIssue;
};

/**
 * 已保存的映射是否还能通过服务端的供应商模型清单校验。
 *
 * 服务端按供应商自己的清单判断接入点模型：供应商下架模型后，留在接入点里的
 * 旧映射会让整次保存被拒绝，且没有任何界面提示。这里用同一份供应商清单提前
 * 找出这些映射，让用户在保存前就能看到并处理。
 */
export function endpointMappingIssue(
  model: EndpointModelLike,
  providers: readonly EndpointMappingProvider[],
): EndpointMappingIssue | null {
  const provider = providers.find(({ id }) => id === model.provider_id);
  if (!provider) {
    return "provider_unavailable";
  }
  // 保存时提交的模型标识和服务端存回的模型 ID 只要有一个还在清单里，
  // 这次保存就不会被清单校验拒绝；两个都不在才算供应商已经下架。
  const identities = [
    model.upstream_model.trim(),
    model.model_name?.trim() ?? "",
  ];
  const providerModels = provider.models ?? [];
  return identities.some((id) => id && providerModels.includes(id))
    ? null
    : "model_removed";
}

export function endpointMappingProblems(
  models: readonly EndpointModelLike[],
  providers: readonly EndpointMappingProvider[],
): EndpointMappingProblem[] {
  return models.flatMap((model) => {
    const issue = endpointMappingIssue(model, providers);
    return issue ? [{ model, issue }] : [];
  });
}

export function endpointMappingIssueLabel(issue: EndpointMappingIssue) {
  return issue === "provider_unavailable" ? "供应商不可用" : "已下架";
}

export function endpointMappingIssueHint(issue: EndpointMappingIssue) {
  return issue === "provider_unavailable"
    ? "该供应商对当前身份不可用，请先移除这个模型映射。"
    : "该供应商已经下架这个模型，请先移除或改选后再保存。";
}

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
        modelGroupName(mapping) === check.upstreamModel,
    )
  ) {
    return { message: "该供应商已经绑定此模型。", title: "模型已存在" };
  }
  return null;
}
