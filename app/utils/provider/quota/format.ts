export function clampPercent(value: number) {
  return Math.max(0, Math.min(100, Number.isFinite(value) ? value : 0));
}

export function formatPercent(value: number) {
  const percent = clampPercent(value);
  return `${percent >= 99.5 ? 100 : Math.round(percent)}%`;
}

export function formatCountdown(value: string) {
  const diff = new Date(value).getTime() - Date.now();
  if (!Number.isFinite(diff) || diff <= 0) return "即将重置";
  const minutes = Math.ceil(diff / 60_000);
  if (minutes < 60) return `${minutes}m 后重置`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    const rest = minutes % 60;
    return rest ? `${hours}h ${rest}m 后重置` : `${hours}h 后重置`;
  }
  const days = Math.floor(hours / 24);
  const rest = hours % 24;
  return rest ? `${days}d ${rest}h 后重置` : `${days}d 后重置`;
}

export function formatUpdatedAt(value: string | null | undefined) {
  if (!value) return "最后更新时间未知";
  return `最后更新 ${new Date(value).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  })}`;
}
