import { invoke } from "@tauri-apps/api/core";
import { computed, readonly, ref, type ComputedRef, type Ref } from "vue";
import { useNotification } from "@stellar/ui";

import { toRelayError, type RelayError } from "~/utils/errors";

export type RelayCommand =
  | "bootstrap"
  | "relay_settings_get"
  | "relay_settings_connect"
  | "relay_settings_save"
  | "desktop_preferences_get"
  | "desktop_preferences_save"
  | "providers_list"
  | "catalog_models_get"
  | "catalog_providers_list"
  | "providers_save"
  | "providers_delete"
  | "providers_ping"
  | "providers_test_protocol"
  | "endpoints_list"
  | "endpoints_save"
  | "endpoints_delete"
  | "endpoints_regenerate_token"
  | "stats_overview"
  | "stats_timeline"
  | "stats_activities"
  | "stats_models"
  | "stats_providers"
  | "stats_leaderboard"
  | "credential_rotate";

export interface CommandState {
  pending: ComputedRef<boolean>;
  error: Ref<RelayError | null>;
}

const managementApiError = ref<RelayError | null>(null);

export function useRelayManagementApiStatus() {
  function clear() {
    managementApiError.value = null;
  }

  return { error: readonly(managementApiError), clear };
}

export function useRelayCommand(): CommandState & {
  invokeCommand<T>(
    command: RelayCommand,
    payload?: Record<string, unknown>,
  ): Promise<T>;
} {
  const notifications = useNotification();
  const pendingRequests = ref(0);
  const pending = computed(() => pendingRequests.value > 0);
  const error = ref<RelayError | null>(null);

  async function invokeCommand<T>(
    command: RelayCommand,
    payload?: Record<string, unknown>,
  ): Promise<T> {
    pendingRequests.value += 1;
    error.value = null;
    managementApiError.value = null;
    try {
      return await invoke<T>(command, payload);
    } catch (caught) {
      const relayError = toRelayError(caught);
      error.value = relayError;
      notifications.danger(notificationMessage(relayError), {
        title: "管理服务请求失败",
      });
      if (relayError.code === "network_error") {
        managementApiError.value = relayError;
      }
      throw relayError;
    } finally {
      pendingRequests.value = Math.max(0, pendingRequests.value - 1);
    }
  }

  return { pending, error, invokeCommand };
}

function notificationMessage(error: RelayError) {
  const status = /^management API returned HTTP (\d{3})(?: .+)?$/.exec(
    error.message,
  );
  const statusCode = status?.[1];
  if (!statusCode) return error.message;

  const explanations: Record<string, string> = {
    "400": "拒绝了请求内容",
    "401": "拒绝了当前设备凭据",
    "403": "拒绝了当前身份的访问",
    "404": "未找到请求的管理接口",
    "405": "不支持本次请求使用的方法",
    "500": "发生内部错误",
  };
  return `管理服务${explanations[statusCode] ?? "拒绝了本次请求"}（HTTP ${statusCode}）。`;
}
