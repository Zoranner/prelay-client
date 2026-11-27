import type { ExtensionCatalogKind } from "~/types/relay";

export function useExtensionUpdates() {
  const extensionCatalog = useExtensionCatalog();
  const { invokeLocalCommand } = useLocalCommand();

  // 扩展页刷新插件前先执行策略同步；规则不参与策略，跳过。
  async function refresh(kind: ExtensionCatalogKind) {
    if (kind === "plugin") {
      try {
        await invokeLocalCommand("extensions_policy_sync", undefined, {
          notify: false,
          trackPending: false,
        });
      } catch {
        // 策略同步失败不阻断目录读取，失败项由策略事件或管理接口状态呈现。
      }
    }
    await extensionCatalog.load(kind, true);
  }

  return { refresh };
}
