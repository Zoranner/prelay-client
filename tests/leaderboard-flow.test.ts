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
  expect(table).toContain("用户排行榜");
  expect(page).not.toContain("estimated_cost");
});
