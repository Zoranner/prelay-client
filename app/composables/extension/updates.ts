import type {
  ExtensionCatalogKind,
  ExtensionPolicyOutcome,
} from "~/types/relay";

export function useExtensionUpdates() {
  const extensionCatalog = useExtensionCatalog();
  const { invokeLocalCommand } = useLocalCommand();
  const policyFeedback = useExtensionPolicyFeedback();

  // 扩展页刷新插件前先执行策略同步；规则不参与策略，跳过。
  async function refresh(kind: ExtensionCatalogKind) {
    if (kind === "plugin") {
      try {
        const outcome = await invokeLocalCommand<ExtensionPolicyOutcome>(
          "extensions_policy_sync",
          undefined,
          {
            notify: false,
            trackPending: false,
          },
        );
        policyFeedback.report(outcome);
      } catch (caught) {
        policyFeedback.reportError(caught);
      }
    }
    await extensionCatalog.load(kind, true);
  }

  return { refresh };
}
