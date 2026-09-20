import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  ExtensionCatalogKind,
  ExtensionCatalogSnapshot,
} from "~/stores/relay";

type CatalogEvent = {
  kind: ExtensionCatalogKind;
  snapshot: ExtensionCatalogSnapshot;
};

// 后端按固定间隔检查规则、Skill、MCP 的更新，只在结果变化时推送对应目录快照；
// 这里负责订阅、落到共享状态，并在启动或服务地址变化时先自行读取一次首帧。
export const EXTENSION_CATALOG_EVENT = "extensions:catalog";

export function useExtensionUpdateStream() {
  const extensionCatalog = useExtensionCatalog();
  let unlisten: UnlistenFn | undefined;

  async function refresh() {
    try {
      await Promise.all(
        extensionCatalogKinds.map((kind) => extensionCatalog.load(kind, true)),
      );
    } catch {
      // 首帧读取失败不打断界面，连接问题由管理接口状态统一呈现。
    }
  }

  async function start() {
    if (!unlisten) {
      unlisten = await listen<CatalogEvent>(EXTENSION_CATALOG_EVENT, (event) =>
        extensionCatalog.applySnapshot(
          event.payload.kind,
          event.payload.snapshot,
        ),
      );
    }
    await refresh();
  }

  function stop() {
    unlisten?.();
    unlisten = undefined;
  }

  return { start, stop };
}
