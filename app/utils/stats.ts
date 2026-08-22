export function parseTimelineBucket(bucket: string): Date | null {
  let value = bucket.trim();
  if (!value) return null;

  if (!value.includes("T")) {
    value = value.includes(" ") ? value.replace(" ", "T") : `${value}T00:00:00`;
  }
  if (!/(?:Z|[+-]\d{2}:?\d{2})$/i.test(value)) {
    value += "+08:00";
  }

  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}
