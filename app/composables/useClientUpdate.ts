import { invoke } from "@tauri-apps/api/core";
import { useNotification } from "@stellar/ui";

import { toRelayError } from "~/utils/errors";

interface DownloadedClientUpdate {
  version: string;
  fileName: string;
}

export type ClientUpdateState =
  "idle" | "checking" | "available" | "downloading" | "ready";

const visible = ref(false);
const version = ref<string | null>(null);
const fileName = ref<string | null>(null);
const state = ref<ClientUpdateState>("idle");
const installing = ref(false);

export function useClientUpdate() {
  const notifications = useNotification();

  async function check() {
    if (state.value === "checking" || state.value === "downloading") return;

    state.value = "checking";
    try {
      const update = await invoke<DownloadedClientUpdate | null>(
        "client_update_prepare",
      );
      if (!update) {
        version.value = null;
        fileName.value = null;
        state.value = "idle";
        return;
      }

      version.value = update.version;
      fileName.value = update.fileName;
      state.value = "available";
    } catch (caught) {
      const error = toRelayError(caught);
      if (error.code !== "client_update_unavailable") {
        notifications.warning(error.message, { title: "更新检查失败" });
      }
      state.value = "idle";
    }
  }

  async function download() {
    if (state.value !== "available" || !version.value || !fileName.value) {
      return;
    }

    state.value = "downloading";
    try {
      await invoke("client_update_prepare", {
        version: version.value,
        fileName: fileName.value,
      });
      state.value = "ready";
      visible.value = true;
    } catch (caught) {
      const error = toRelayError(caught);
      notifications.warning(error.message, { title: "更新下载失败" });
      state.value = "available";
    }
  }

  function openInstallDialog() {
    if (state.value === "ready") visible.value = true;
  }

  async function install() {
    if (!version.value || !fileName.value || installing.value) return;

    installing.value = true;
    try {
      await invoke("client_update_install", {
        version: version.value,
        fileName: fileName.value,
      });
    } catch (caught) {
      const error = toRelayError(caught);
      notifications.danger(error.message, { title: "启动安装程序失败" });
      installing.value = false;
    }
  }

  return {
    check,
    download,
    installing,
    install,
    openInstallDialog,
    state,
    version,
    visible,
  };
}
