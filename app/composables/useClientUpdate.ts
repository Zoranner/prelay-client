import { invoke } from "@tauri-apps/api/core";
import { useNotification } from "stellar-ui";

import { toRelayError } from "~/utils/errors";

interface DownloadedClientUpdate {
  version: string;
  fileName: string;
}

const visible = ref(false);
const version = ref<string | null>(null);
const fileName = ref<string | null>(null);
const preparing = ref(false);
const installing = ref(false);

export function useClientUpdate() {
  const notifications = useNotification();

  async function prepare() {
    if (preparing.value || visible.value) return;

    preparing.value = true;
    try {
      const update = await invoke<DownloadedClientUpdate | null>(
        "client_update_prepare",
      );
      if (!update) return;

      version.value = update.version;
      fileName.value = update.fileName;
      visible.value = true;
    } catch (caught) {
      const error = toRelayError(caught);
      if (error.code !== "client_update_unavailable") {
        notifications.warning(error.message, { title: "更新检查失败" });
      }
    } finally {
      preparing.value = false;
    }
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

  return { installing, prepare, install, version, visible };
}
