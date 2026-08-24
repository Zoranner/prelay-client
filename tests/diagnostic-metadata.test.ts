import { expect, test } from "bun:test";

import { requestDiagnostics } from "../app/utils/diagnosticMetadata";

test("诊断元数据提取结构化诊断和流式问题", () => {
  expect(
    requestDiagnostics(
      '{"diagnostics":[{"code":"usage_missing","message":"缺少用量","count":2,"severity":"warning"}],"stream":{"completed":false}}',
    ),
  ).toEqual({
    diagnostics: [
      {
        action: "-",
        code: "usage_missing",
        count: 2,
        message: "缺少用量",
        paths: [],
        severity: "warning",
      },
    ],
    streamIssue: "流式响应未完整结束",
  });
});

test("无效或无诊断的元数据不会打开请求诊断", () => {
  expect(requestDiagnostics("not-json")).toBeNull();
  expect(requestDiagnostics('{"trace":"request-1"}')).toBeNull();
  expect(requestDiagnostics(null)).toBeNull();
});
