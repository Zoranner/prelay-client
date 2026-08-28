export interface RequestDiagnostic {
  action: string;
  code: string;
  count: number;
  message: string;
  paths: string[];
  severity: "info" | "warning";
}

export interface RequestDiagnostics {
  diagnostics: RequestDiagnostic[];
  streamIssue: string | null;
}

export function requestDiagnostics(
  metadata: string | null,
): RequestDiagnostics | null {
  if (!metadata?.trim()) return null;

  try {
    const parsed = JSON.parse(metadata);
    if (!isRecord(parsed)) return null;

    const diagnostics = Array.isArray(parsed.diagnostics)
      ? parsed.diagnostics.flatMap(parseDiagnostic)
      : [];
    const streamIssue = parseStreamIssue(parsed.stream);
    return diagnostics.length || streamIssue
      ? { diagnostics, streamIssue }
      : null;
  } catch {
    return null;
  }
}

function parseDiagnostic(value: unknown): RequestDiagnostic[] {
  if (!isRecord(value)) return [];

  const code = stringValue(value.code);
  const message = stringValue(value.message);
  if (!code || !message) return [];

  return [
    {
      action: stringValue(value.action) ?? "-",
      code,
      count: positiveInteger(value.count) ?? 1,
      message,
      paths: Array.isArray(value.paths)
        ? value.paths.flatMap((path) => {
            const value = stringValue(path);
            return value ? [value] : [];
          })
        : [],
      severity: value.severity === "warning" ? "warning" : "info",
    },
  ];
}

function parseStreamIssue(value: unknown): string | null {
  if (!isRecord(value)) return null;
  if (value.empty === true) return "上游流未返回任何数据";
  if (value.completed === false) return "流式响应未完整结束";
  if (value.final_usage_seen === false) return "流式响应未返回最终用量";
  return null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function positiveInteger(value: unknown): number | null {
  return typeof value === "number" && Number.isInteger(value) && value > 0
    ? value
    : null;
}

function stringValue(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value : null;
}
