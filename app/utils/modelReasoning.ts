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
  if (custom) return genericReasoningEffortOptions.map((option) => ({ ...option }));

  const efforts = model?.reasoning_efforts ?? [];
  if (efforts.length === 0) return [];

  const defaultEffort = model?.default_reasoning_effort ?? "未指定";
  return [
    { value: "", label: `跟随模型默认（${defaultEffort}）` },
    ...efforts.map((value) => ({
      value,
      label: reasoningEffortLabel(value),
    })),
  ];
}

export function normalizeReasoningEffort(
  value: string,
  model: CatalogLanguageModelResponse | undefined,
): string {
  if (!value) return "";
  return model?.reasoning_efforts?.includes(value) ? value : "";
}
