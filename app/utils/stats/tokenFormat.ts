export function formatTokens(value: number) {
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toLocaleString("zh-CN");
}

export function formatTrendValue(value: number, percent = false) {
  if (!Number.isFinite(value)) return "-";
  if (percent) return `${Number(value.toFixed(1))}%`;
  return formatTokens(value);
}
