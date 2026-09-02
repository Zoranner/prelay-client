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
  expect(table).toContain('import { Avatar, Card } from "@stellar/ui"');
  expect(table).not.toContain("<List");
  expect(table).not.toContain("ListItem");
  expect(table).toContain("<ol");
  expect(table).toContain('<Card full-height class="user-leaderboard-card"');
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
  expect(table).not.toContain("rows.length");
  expect(table).not.toContain(" 人");
  expect(table).toContain("import { Avatar as DiceBearAvatar, Style }");
  expect(table).toContain(
    'import cutouts from "@dicebear/styles/cutouts.json"',
  );
  expect(table).toContain("identity_id");
  expect(table).toContain(':src="avatarSrc(row)"');
  expect(table).not.toContain('class="leaderboard-avatar-wrap"');
  expect(table).not.toContain("position: absolute");
  expect(table).toContain("grid-template-columns: 40px minmax(0, 1fr) 24px");
  expect(table).toContain("text-align: right");
  expect(table).toContain("align-self: center");
  expect(page).toContain('class="dashboard-primary-grid"');
  expect(page).not.toContain("current-avatar-seed");
  expect(page).not.toContain("current-display-name");
  expect(page).toContain("grid-template-columns: repeat(6, minmax(0, 1fr))");
  expect(page).toContain("grid-template-columns: repeat(3, minmax(0, 1fr))");
  expect(page).toContain("grid-template-columns: repeat(2, minmax(0, 1fr))");
  expect(page).toContain("grid-template-columns: 1fr");
  expect(page).not.toContain("estimated_cost");
});
