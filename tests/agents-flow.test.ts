import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );

test("智能体页自动识别本机 Codex 与 Claude Code 扩展", () => {
  const navigation = source("components/dashboard/DashboardShell.vue");
  const page = source("pages/agents.vue");
  const list = source("components/agents/AgentItemList.vue");

  expect(navigation).toContain('label: "智能体"');
  expect(navigation).toContain('path: "/agents"');
  expect(page).toContain('"agents_list"');
  expect(page).toContain("workspaceExit.register");
  expect(page).toContain("void loadAgentPage()");
  expect(page).toContain("刷新");
  expect(page).toContain('from "~/components/agents/AgentItemList.vue"');
  expect(page).toContain("@lobehub/icons-static-svg");
  expect(page).toContain("activeClient");
  expect(page).toContain("availableClients");
  expect(page).toContain("snapshot.value.clients");
  expect(page).toContain('client.version ?? "版本未知"');
  expect(page).not.toContain('const clients: Array<{ client: AgentClient;');
  expect(page).toContain("未检测到已安装的智能体");
  expect(page).toContain("activeKind");
  expect(page).toContain('value: "plugin"');
  expect(page).toContain('value: "mcp"');
  expect(page).toContain('value: "skill"');
  expect(page).toContain("useConfirm");
  expect(page).toContain("uninstallAgentItem");
  expect(page).toContain('"agents_remove"');
  expect(page).toContain('@uninstall="uninstallAgentItem"');
  expect(page).toContain('<List class="agent-client-list"');
  expect(page).toContain("<ListItem");
  expect(page).toContain(':active="activeClient === client.client"');
  expect(page).toContain('v-model="activeSection"');
  expect(page).toContain('variant="button"');
  expect(page).not.toContain("item-toolbar");
  expect(page).not.toContain("<Sidebar");
  expect(page).not.toContain("<Tabs");
  expect(page).not.toContain(':extra="String(client.items.length)"');
  expect(page).toContain("agent-client-icon--monochrome");
  expect(page).toContain(".agent-client-icon {\n  width: 24px;");
  expect(page).toContain("filter: var(--pr-monochrome-icon-filter)");
  expect(source("assets/css/main.css")).toContain("html.light");
  expect(source("assets/css/main.css")).toContain(
    "--pr-monochrome-icon-filter: brightness(0)",
  );
  expect(list).toContain("启用");
  expect(list).toContain("禁用");
  expect(list).toContain("错误");
  expect(list).toContain("Badge, Button, Table, useNotification");
  expect(list).toContain("<Badge :variant=\"statusVariant(row.status)\">");
  expect(list).not.toContain("<Tag");
  expect(list).toContain('class="agent-item-table"');
  expect(list).toContain("fixed-header");
  expect(list).toContain('layout="fixed"');
  expect(list.indexOf('key: "version"')).toBeLessThan(
    list.indexOf('key: "status"'),
  );
  expect(list.indexOf('key: "status"')).toBeLessThan(
    list.indexOf('key: "sourcePath"'),
  );
  expect(list).toContain("<template #cell-sourcePath");
  expect(list).toContain('icon="ph:copy"');
  expect(list).toContain("copySourcePath(row.sourcePath)");
  expect(list).toContain('key: "actions"');
  expect(list).toContain('icon="ph:trash"');
  expect(list).toContain("emit('uninstall', row)");
  expect(list).toContain("text-overflow: ellipsis");
  expect(list).toContain("overflow-x: auto");
  expect(list).not.toContain("item-group-header");
  expect(list).not.toContain("@lobehub/icons-static-svg");
  expect(page).toContain(".item-results {\n  display: flex;");
  expect(page).toContain("overflow: hidden");
  expect(page).not.toContain("overflow: auto");
  expect(list).not.toContain("http");
});

test("智能体页的本地 command 不复用管理服务命令状态", () => {
  const page = source("pages/agents.vue");
  const localCommand = source("composables/useLocalCommand.ts");
  const relayCommand = source("composables/useRelayCommand.ts");

  expect(page).toContain("useLocalCommand");
  expect(localCommand).toContain('from "@tauri-apps/api/core"');
  expect(localCommand).toContain('"agents_remove"');
  expect(relayCommand).not.toContain('"agents_list"');
});
