import { useConfirm, useNotification } from "@stellar/ui";
import type {
  AgentClient,
  ExtensionCatalogPackage,
  ExtensionInstallResult,
} from "~/types/relay";
import { errorText, toRelayError } from "~/utils/errors";
import {
  runUpdateQueue,
  type UpdateQueueResult,
} from "~/utils/extension/updateQueue";

type InstallJob = {
  label: string;
  running: boolean;
};

type InstallRequest = {
  package: ExtensionCatalogPackage;
  clients: AgentClient[];
};

function busyLabel(extension: ExtensionCatalogPackage) {
  if (extension.installAction === "update") return "更新中";
  if (extension.installAction === "partial") return "补装中";
  return "安装中";
}

function isMcpExtension(extension: ExtensionCatalogPackage) {
  return extension.artifacts.some((artifact) => artifact.kind === "mcp");
}

function isRuleExtension(extension: ExtensionCatalogPackage) {
  return extension.catalogKind === "rule";
}

export function useExtensionInstallJobs() {
  const jobs = useState<Record<string, InstallJob>>(
    "extension-install-jobs",
    () => ({}),
  );
  const batchUpdating = useState<boolean>(
    "extension-update-batch-running",
    () => false,
  );
  const { invokeLocalCommand } = useLocalCommand();
  const extensionCatalog = useExtensionCatalog();
  const notifications = useNotification();
  const { confirm: confirmAction } = useConfirm();
  const activeJobs = computed<Record<string, InstallJob>>(() =>
    Object.fromEntries(
      Object.entries(jobs.value).map(([name, job]) => [
        name,
        { label: job.label, running: job.running },
      ]),
    ),
  );

  async function run(request: InstallRequest, overwrite: boolean) {
    return invokeLocalCommand<ExtensionInstallResult>(
      "extensions_install",
      {
        request: {
          package: request.package,
          clients: request.clients,
          overwrite,
        },
      },
      { notify: false },
    );
  }

  async function confirmOverwrite(extension: ExtensionCatalogPackage) {
    if (isRuleExtension(extension)) {
      return confirmAction({
        title: "覆盖本机全局规则",
        message: "本机已有全局规则内容，是否覆盖安装？",
        description:
          "覆盖会用当前规则包内容替换本机全局规则文档，现有内容不会保留。",
        confirmText: "覆盖",
        danger: true,
      });
    }
    return confirmAction({
      title: isMcpExtension(extension) ? "覆盖已有 MCP 配置" : "覆盖已有技能",
      message: isMcpExtension(extension)
        ? "目标 MCP 服务名已存在，是否覆盖安装？"
        : "目标技能目录已存在，是否覆盖安装？",
      description: isMcpExtension(extension)
        ? "覆盖会替换目标 MCP 服务配置，并更新当前扩展包的版本记录。"
        : "覆盖会删除该技能目录中的现有文件，然后写入当前扩展包内容。",
      confirmText: "覆盖",
      danger: true,
    });
  }

  async function refreshCatalogKind(
    kind: ExtensionCatalogPackage["catalogKind"],
  ) {
    try {
      await extensionCatalog.load(kind, true);
    } catch {
      // 目录刷新失败由列表空态和管理接口状态呈现。
    }
  }

  async function refreshCatalog(extension: ExtensionCatalogPackage) {
    await refreshCatalogKind(extension.catalogKind);
  }

  function setJob(
    extension: ExtensionCatalogPackage,
    label: string,
    running: boolean,
  ) {
    jobs.value = {
      ...jobs.value,
      [extension.name]: { label, running },
    };
  }

  function removeJob(name: string) {
    if (!jobs.value[name]) return;
    const next = { ...jobs.value };
    delete next[name];
    jobs.value = next;
  }

  async function install(request: InstallRequest) {
    const name = request.package.name;
    if (jobs.value[name]) return;
    setJob(request.package, busyLabel(request.package), true);
    try {
      let result: ExtensionInstallResult;
      try {
        result = await run(request, false);
      } catch (caught) {
        const error = caught as { code?: string };
        if (error.code !== "extension_target_exists") throw caught;
        if (!(await confirmOverwrite(request.package))) return;
        result = await run(request, true);
      }
      if (result.warning) {
        notifications.warning(result.warning, { title: "Hook 信任未写入" });
      }
      notifications.success(result.message || "扩展已安装");
      await refreshCatalog(request.package);
    } catch (caught) {
      notifications.error(errorText(toRelayError(caught)), {
        title: "扩展安装失败",
      });
      await refreshCatalog(request.package);
    } finally {
      removeJob(name);
    }
  }

  function notifyUpdateResult(
    results: UpdateQueueResult<ExtensionCatalogPackage>[],
    warnings: string[],
  ) {
    const failures = results.filter((result) => result.error);
    const updated = results.length - failures.length;
    if (failures.length) {
      const names = failures.map((result) => result.item.name).join("、");
      const firstFailure = failures[0];
      const detail = firstFailure
        ? errorText(toRelayError(firstFailure.error))
        : "";
      const message = `${names} 更新失败。${detail}`;
      if (updated > 0) {
        notifications.warning(message, { title: "部分扩展更新失败" });
      } else {
        notifications.error(message, { title: "扩展更新失败" });
      }
    } else if (updated > 0) {
      notifications.success(`已更新 ${updated} 个扩展。`);
    }

    if (warnings.length) {
      notifications.warning([...new Set(warnings)].join(" "), {
        title: "Hook 信任未写入",
      });
    }
  }

  async function updateAll(extensions: ExtensionCatalogPackage[]) {
    if (batchUpdating.value) return;
    const targets = extensions.filter(
      (extension) =>
        extension.installAction === "update" && !jobs.value[extension.name],
    );
    if (!targets.length) return;

    batchUpdating.value = true;
    const next = { ...jobs.value };
    for (const extension of targets) {
      next[extension.name] = {
        label: "等待中",
        running: false,
      };
    }
    jobs.value = next;

    const warnings: string[] = [];
    try {
      const results = await runUpdateQueue(
        targets,
        async (extension) => {
          try {
            const result = await run(
              {
                package: extension,
                clients: extension.installedClients,
              },
              false,
            );
            if (result.warning) warnings.push(result.warning);
          } finally {
            await refreshCatalog(extension);
            removeJob(extension.name);
          }
        },
        {
          onStart: (extension) => {
            setJob(extension, "更新中", true);
          },
        },
      );
      await Promise.all(
        [...new Set(targets.map((extension) => extension.catalogKind))].map(
          refreshCatalogKind,
        ),
      );
      notifyUpdateResult(results, warnings);
    } catch (caught) {
      notifications.error(errorText(toRelayError(caught)), {
        title: "扩展更新失败",
      });
    } finally {
      for (const extension of targets) removeJob(extension.name);
      batchUpdating.value = false;
    }
  }

  return { activeJobs, batchUpdating, install, updateAll };
}
