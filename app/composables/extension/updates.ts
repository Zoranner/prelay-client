import type { ExtensionCatalogKind } from "~/types/relay";

export function useExtensionUpdates() {
  const extensionCatalog = useExtensionCatalog();
  const { invokeLocalCommand } = useLocalCommand();

  // 扩展页刷新插件前先执行策略同步；规则不参与策略，跳过。
  async function refresh(kind: ExtensionCatalogKind) {
    if (kind === "plugin") {
      await invokeLocalCommand("extensions_policy_sync", undefined, {
        notify: false,
        trackPending: false,
      });
    }
    await extensionCatalog.load(kind, true);
  }

  return { refresh };
}
