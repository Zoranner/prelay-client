import type { ExtensionCatalogKind } from "~/types/relay";

export function useExtensionUpdates() {
  const extensionCatalog = useExtensionCatalog();

  // 扩展页只重读目录；智能体页在进入或切换客户端时重新扫描本机内容。
  async function refresh(kind: ExtensionCatalogKind) {
    await extensionCatalog.load(kind, true);
  }

  return { refresh };
}
