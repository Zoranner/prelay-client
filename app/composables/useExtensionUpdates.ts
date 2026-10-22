import type { ExtensionCatalogKind } from "~/stores/relay";

import { useNotification } from "@stellar/ui";

export function useExtensionUpdates({
  refreshClient,
}: {
  refreshClient: () => Promise<void>;
}) {
  const extensionCatalog = useExtensionCatalog();
  const { invokeLocalCommand } = useLocalCommand();
  const notifications = useNotification();
  const updating = ref(false);

  // 规则写入本机规则文件，插件按制品写入条目目录，
  // 都需要同时重读分类目录与该客户端的本机内容。
  async function refresh(kind: ExtensionCatalogKind) {
    await Promise.all([extensionCatalog.load(kind, true), refreshClient()]);
  }

  async function installed(kind: ExtensionCatalogKind) {
    await refresh(kind);
    notifications.success("扩展已安装");
  }

  async function updateAll(kind: ExtensionCatalogKind) {
    if (updating.value) return;
    updating.value = true;
    try {
      const result = await invokeLocalCommand<{ message: string }>(
        "extensions_update_all",
        { kind },
        { notify: false },
      );
      await refresh(kind);
      notifications.success(result.message);
    } finally {
      updating.value = false;
    }
  }

  return { installed, refresh, updateAll, updating };
}
