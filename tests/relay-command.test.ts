import { expect, mock, test } from "bun:test";

import { errorReason, errorText, toRelayError } from "../app/utils/errors";

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (reason: unknown) => void;
};

const requests: Deferred<unknown>[] = [];
const notifications: Array<{ message: string; title?: string }> = [];

mock.module("@tauri-apps/api/core", () => ({
  invoke: () => {
    let resolve!: (value: unknown) => void;
    let reject!: (reason: unknown) => void;
    const promise = new Promise<unknown>((settle, fail) => {
      resolve = settle;
      reject = fail;
    });
    requests.push({ promise, resolve, reject });
    return promise;
  },
}));

mock.module("@stellar/ui", () => ({
  useNotification: () => ({
    error: (message: string, options?: { title?: string }) => {
      notifications.push({ message, title: options?.title });
      return notifications.length;
    },
  }),
}));

mock.module("~/utils/errors", () => ({
  errorReason,
  errorText,
  toRelayError,
}));

const { useRelayCommand, useRelayManagementApiStatus } =
  await import("../app/composables/useRelayCommand");

test("并行 command 在最后一个请求结束前保持 pending", async () => {
  requests.length = 0;
  const relay = useRelayCommand();
  const first = relay.invokeCommand("stats_overview");
  const second = relay.invokeCommand("stats_models");

  expect(relay.pending.value).toBe(true);
  requests[0]?.resolve({});
  await first;

  expect(relay.pending.value).toBe(true);
  requests[1]?.resolve([]);
  await second;
  expect(relay.pending.value).toBe(false);
});

test("管理 API 不可达时公开全局阻断状态", async () => {
  requests.length = 0;
  const relay = useRelayCommand();
  const managementApi = useRelayManagementApiStatus();
  const request = relay.invokeCommand("stats_overview");

  requests[0]?.reject({
    code: "network_error",
    message: "unable to reach the relay management API",
  });

  await expect(request).rejects.toEqual({
    code: "network_error",
    message: "unable to reach the relay management API",
  });
  expect(managementApi.error.value?.code).toBe("network_error");
});

test("校验失败的通知直接展示服务端给出的具体原因", async () => {
  requests.length = 0;
  notifications.length = 0;
  const relay = useRelayCommand();
  const request = relay.invokeCommand("endpoints_save");

  requests[0]?.reject({
    code: "validation_failed",
    message:
      "模型 gpt-6-luna 不在供应商“GoToken 套餐”的模型清单里，请先移除或改选后再保存",
  });

  await expect(request).rejects.toMatchObject({ code: "validation_failed" });
  expect(notifications).toEqual([
    {
      message:
        "模型 gpt-6-luna 不在供应商“GoToken 套餐”的模型清单里，请先移除或改选后再保存",
      title: "管理服务请求失败",
    },
  ]);
});

test("没有具体原因的失败仍然使用固定文案", async () => {
  requests.length = 0;
  notifications.length = 0;
  const relay = useRelayCommand();
  const request = relay.invokeCommand("stats_overview");

  requests[0]?.reject({ code: "internal", message: "Internal server error" });

  await expect(request).rejects.toMatchObject({ code: "internal" });
  expect(notifications).toEqual([
    { message: "服务端发生内部错误。", title: "管理服务请求失败" },
  ]);
});
