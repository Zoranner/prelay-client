import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

test("仪表盘通过 Tauri 展示全用户活动排行榜", () => {
  const commands = read("app/composables/useRelayCommand.ts");
  const nativeStats = read("src-tauri/src/commands/stats.rs");
  const nativeRegistration = read("src-tauri/src/app/mod.rs");
  const store = read("app/stores/relay.ts");
  const page = read("app/pages/index.vue");
  const table = read("app/components/dashboard/UserLeaderboardTable.vue");

  expect(commands).toContain('"stats_leaderboard"');
  expect(nativeStats).toContain("pub async fn stats_leaderboard");
  expect(nativeRegistration).toContain(
    "crate::commands::stats::stats_leaderboard",
  );
  expect(store).toContain("export interface UserLeaderboardEntry");
  expect(page).toContain("UserLeaderboardEntry");
  expect(page).toContain('"stats_leaderboard"');
  expect(page).toContain('metric: "total_tokens"');
  expect(table).toContain("用户排行榜");
  expect(table).toContain(
    'import { Avatar, Card, Skeleton } from "@stellar/ui"',
  );
  expect(table).not.toContain("<List");
  expect(table).not.toContain("ListItem");
  expect(table).toContain("<ol");
  expect(table).toContain('<Card class="user-leaderboard-card"');
  expect(table).not.toContain("overflow-y: auto");
  expect(table).not.toContain("<Table");
  expect(table).not.toContain("const columns");
  expect(table).not.toContain("success_rate");
  expect(table).not.toContain("formatRate");
  expect(table).not.toContain("activity_count");
  expect(table).not.toContain("次活动");
  expect(table).toContain("总 Token");
  expect(table).toContain(".leaderboard-entry");
  expect(table).toContain("display: grid");
  expect(table).toContain("justify-self: start");
  expect(table).toContain("justify-self: end");
  // 卡片不展示上榜人数（个人视角只报自己的名次）
  expect(table).not.toContain("共 ");
  expect(table).not.toContain(" 人");
  expect(table).toContain(
    'import { identityAvatarSrc } from "~/utils/identityAvatar"',
  );
  expect(table).toContain("identity_id");
  expect(table).toContain(':src="identityAvatarSrc(row.identity_id)"');
  expect(table).not.toContain('class="leaderboard-avatar-wrap"');
  expect(table).not.toContain("position: absolute");
  expect(table).toContain("grid-template-columns: 40px minmax(0, 1fr) 24px");
  expect(table).toContain("text-align: right");
  expect(table).toContain("align-self: center");
  expect(table).toContain("container-type: inline-size");
  expect(table).toContain("@container (max-width: 260px)");
  expect(table).toContain("@container (max-width: 200px)");
  expect(table).toContain("grid-template-columns: 40px minmax(0, 1fr) 18px");
  expect(table).toContain("white-space: nowrap");
  expect(page).toContain('class="dashboard-main"');
  expect(page).toContain('class="dashboard-rail"');
  expect(page).not.toContain("current-avatar-seed");
  expect(page).not.toContain("current-display-name");
  expect(page).toContain(
    "grid-template-columns: minmax(0, 1fr) clamp(260px, 24vw, 320px)",
  );
  expect(page).not.toContain("grid-column: span");
  expect(page).toContain("grid-template-columns: 1fr");
  expect(page).not.toContain("estimated_cost");
  expect(page).toContain("<ModelLeaderboard");
  expect(page).toContain(
    '<UserLeaderboardTable :rows="leaderboardRows" :loading="pending" />',
  );
  expect(page).toContain(
    '<ModelLeaderboard :rows="models" :loading="pending" />',
  );
  expect(page).not.toContain("<ActivityHeatmap");
});
