import type { CatalogLanguageModelResponse } from "~/stores/relay";

export type ReasoningEffortOption = {
  value: string;
  label: string;
  description?: string;
};

const genericReasoningEffortOptions: ReasoningEffortOption[] = [
  { value: "low", label: "低" },
  { value: "medium", label: "中" },
  { value: "high", label: "高" },
  { value: "xhigh", label: "很高" },
];

const reasoningEffortLabels: Record<string, string> = {
  none: "无",
  minimal: "极低",
  low: "低",
  medium: "中",
  high: "高",
  xhigh: "很高",
  max: "最高",
};

function reasoningEffortLabel(value: string) {
  return reasoningEffortLabels[value] ?? value;
}

export function reasoningEffortOptions(
  model: CatalogLanguageModelResponse | undefined,
  custom: boolean,
): ReasoningEffortOption[] {
  if (custom)
    return genericReasoningEffortOptions.map((option) => ({ ...option }));

  const efforts = model?.reasoning_efforts ?? [];
  if (efforts.length === 0) return [];

  return efforts.map((value) => ({
    value,
    label: reasoningEffortLabel(value),
  }));
}

export function defaultReasoningEffort(
  model: CatalogLanguageModelResponse | undefined,
): string {
  const defaultEffort = model?.default_reasoning_effort;
  return defaultEffort && model.reasoning_efforts?.includes(defaultEffort)
    ? defaultEffort
    : "";
}

export function normalizeReasoningEffort(
  value: string,
  model: CatalogLanguageModelResponse | undefined,
): string {
  const normalized = value.trim();
  if (normalized && model?.reasoning_efforts?.includes(normalized)) {
    return normalized;
  }
  return defaultReasoningEffort(model);
}
