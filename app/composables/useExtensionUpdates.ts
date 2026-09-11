import { useNotification } from "@stellar/ui";

export function useExtensionUpdates({
  refreshSkills,
}: {
  refreshSkills: () => Promise<void>;
}) {
  const extensionCatalog = useExtensionCatalog();
  const { invokeLocalCommand } = useLocalCommand();
  const notifications = useNotification();
  const updating = ref(false);

  async function installed(kind: "rule" | "skill" | "mcp") {
    const refresh = [extensionCatalog.load(kind, true)];
    if (kind === "skill") refresh.push(refreshSkills());
    await Promise.all(refresh);
    notifications.success("扩展已安装");
  }

  async function updateAll() {
    if (updating.value) return;
    updating.value = true;
    try {
      const result = await invokeLocalCommand<{ message: string }>(
        "extensions_update_all",
        {},
        { notify: false },
      );
      await Promise.all([refreshSkills(), extensionCatalog.load("skill", true)]);
      notifications.success(result.message);
    } finally {
      updating.value = false;
    }
  }

  return { installed, updateAll, updating };
}
