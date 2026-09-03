import type {
  CatalogModelResponse,
  CatalogProvider,
  CatalogProviderProtocol,
  UpstreamProtocol,
} from "~/stores/relay";
import { modelCatalogEntry, modelCatalogLabel } from "~/utils/modelCatalog";

export type ProviderTemplate = {
  value: string;
  label: string;
  providerType: string;
  baseUrl: string;
  protocols: UpstreamProtocol[];
  protocolBaseUrls: Partial<Record<UpstreamProtocol, string>>;
  languageModels: string[];
  imageGenerationModels: string[];
  models: string[];
  custom?: boolean;
};

export type ProviderModelOption = {
  value: string;
  label: string;
  model: CatalogModelResponse | undefined;
};

export function providerTemplates(catalogProviders: CatalogProvider[]) {
  return catalogProviders.map(catalogProviderTemplate);
}

export function providerTemplateForType(
  providerType: string,
  catalogProviders: CatalogProvider[] = [],
) {
  return providerTemplates(catalogProviders).find(
    (item) => item.providerType === providerType,
  );
}

export function catalogProviderTemplate(
  provider: CatalogProvider,
): ProviderTemplate {
  const protocolBaseUrls: Partial<Record<UpstreamProtocol, string>> = {};
  for (const entry of provider.protocol_base_urls) {
    protocolBaseUrls[upstreamProtocol(entry.protocol)] = entry.base_url;
  }

  return {
    value: `catalog:${provider.id}`,
    label: provider.name,
    providerType: provider.id,
    baseUrl: provider.base_url,
    protocols: provider.protocols.map(upstreamProtocol),
    protocolBaseUrls,
    languageModels: provider.language_models,
    imageGenerationModels: provider.image_generation_models,
    models: [...provider.language_models, ...provider.image_generation_models],
  };
}

export function providerModelOptions(
  modelIds: string[],
): ProviderModelOption[] {
  return modelIds.map((id) => ({
    value: id,
    label: modelCatalogLabel(id),
    model: modelCatalogEntry(id),
  }));
}

export type ProtocolTagVariant = "primary" | "success" | "warning" | "default";

export function protocolLabel(protocol: string | null) {
  return protocol === "responses"
    ? "Responses"
    : protocol === "anthropic" || protocol === "anthropic_messages"
      ? "Anthropic Messages"
      : protocol === "openai" || protocol === "chat_completions"
        ? "Chat Completions"
        : protocol === "images_generations"
          ? "Images Generations"
          : "-";
}

export function protocolTagVariant(
  protocol: string | null,
): ProtocolTagVariant {
  return protocol === "responses"
    ? "success"
    : protocol === "anthropic" || protocol === "anthropic_messages"
      ? "warning"
      : protocol === "openai" || protocol === "chat_completions"
        ? "primary"
        : "default";
}

function upstreamProtocol(protocol: CatalogProviderProtocol): UpstreamProtocol {
  if (protocol === "chat_completions") return "openai";
  if (protocol === "anthropic_messages") return "anthropic";
  return protocol;
}
