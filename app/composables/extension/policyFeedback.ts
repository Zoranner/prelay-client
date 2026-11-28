import { useNotification } from "@stellar/ui";

import type { ExtensionPolicyOutcome } from "~/types/relay";
import { errorText, toRelayError } from "~/utils/errors";

export function useExtensionPolicyFeedback() {
  const notifications = useNotification();

  function report(outcome: ExtensionPolicyOutcome) {
    if (outcome.failed.length) {
      notifications.error(
        outcome.failed
          .map((item) => `${item.name}：${item.message}`)
          .join("；"),
        { title: "扩展策略同步失败" },
      );
    }
    if (outcome.warnings.length) {
      notifications.warning([...new Set(outcome.warnings)].join(" "), {
        title: "扩展策略同步警告",
      });
    }
    if (
      !outcome.failed.length &&
      (outcome.installed.length || outcome.uninstalled.length)
    ) {
      const actions: string[] = [];
      if (outcome.installed.length) {
        actions.push(`安装 ${outcome.installed.length} 个`);
      }
      if (outcome.uninstalled.length) {
        actions.push(`卸载 ${outcome.uninstalled.length} 个`);
      }
      const hooks = outcome.review.length
        ? `，写入 ${outcome.review.length} 个 Hook`
        : "";
      notifications.success(
        `扩展策略同步完成：${actions.join("，")}${hooks}。`,
      );
    }
  }

  function reportError(caught: unknown) {
    notifications.error(errorText(toRelayError(caught)), {
      title: "扩展策略同步失败",
    });
  }

  return { report, reportError };
}
