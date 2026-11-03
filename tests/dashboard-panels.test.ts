import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path: string) =>
  readFileSync(new URL(`../app/${path}`, import.meta.url), "utf8");

test("模型排行榜按 Token 取前五并画条形", () => {
  const leaderboard = source("components/dashboard/leaderboard/Models.vue");

  expect(leaderboard).toContain(
    'import { Card, EmptyState, Skeleton } from "@stellar/ui"',
  );
  expect(leaderboard).toContain("模型排行榜");
  expect(leaderboard).toContain("Skeleton");
  expect(leaderboard).toContain("loading?: boolean");
  expect(leaderboard).toContain('v-if="props.loading && !entries.length"');
  expect(leaderboard).toContain("按 Token");
  expect(leaderboard).not.toContain("statsScopeLabel");
  expect(leaderboard).toContain(
    "tokens: (row.input_tokens ?? 0) + (row.output_tokens ?? 0)",
  );
  expect(leaderboard).toContain(".slice(0, props.limit)");
  expect(leaderboard).toContain(
    '<i :style="{ width: barWidth(entry.tokens) }" />',
  );
  expect(leaderboard).toContain("formatTokens(entry.tokens)");
  expect(leaderboard).toContain("暂无模型数据");
});

test("仪表盘隐藏活跃度，右栏放模型与用户排行", () => {
  const page = source("pages/index.vue");

  expect(page).toContain('class="dashboard-main"');
  expect(page).toContain('class="dashboard-rail"');
  expect(page).toContain(
    "grid-template-columns: minmax(0, 1fr) clamp(260px, 24vw, 320px)",
  );
  expect(page).toContain(
    '<UserLeaderboardTable :rows="leaderboardRows" :loading="pending" />',
  );
  expect(page).toContain(
    '<ModelLeaderboard :rows="models" :loading="pending" />',
  );
  // 右栏顺序：用户排行榜、模型排行榜
  expect(page.indexOf("<UserLeaderboardTable")).toBeLessThan(
    page.indexOf("<ModelLeaderboard"),
  );
});

test("仪表盘提供个人与团队视角切换", () => {
  const page = source("pages/index.vue");
  const scopeSwitch = source("components/dashboard/stats/ScopeSwitch.vue");
  const leaderboard = source("components/dashboard/leaderboard/Users.vue");
  expect(leaderboard).toContain("Skeleton");
  expect(leaderboard).toContain("loading?: boolean");
  expect(leaderboard).toContain('v-if="props.loading && !props.rows.length"');
  expect(leaderboard).toMatch(
    /\.leaderboard-avatar\s*\{[\s\S]*width: 40px;[\s\S]*height: 40px;/,
  );
  expect(leaderboard).not.toContain("width: 32px;\n    height: 32px;");
  expect(leaderboard).not.toContain("width: 28px;\n    height: 28px;");
  const stats = source("utils/stats/bucket.ts");

  expect(page).toContain('const statsScope = ref<StatsScope>("personal");');
  expect(page).toContain('<StatsScopeSwitch v-model="statsScope" />');
  expect(page).toContain("<template #header-inline>");
  expect(scopeSwitch).toContain('{ label: "个人", value: "personal" }');
  expect(scopeSwitch).toContain('{ label: "团队", value: "team" }');
  expect(scopeSwitch).toContain("<button");
  expect(scopeSwitch).toContain("border-radius: 999px");
  expect(scopeSwitch).toContain("stats-scope-switch__option--active");
  expect(scopeSwitch).toContain("background: var(--st-primary)");
  expect(scopeSwitch).not.toContain("@stellar/ui");
  expect(page).toContain(
    "const query = { range: selectedRange.value, scope: statsScope.value };",
  );
  expect(page).toContain("watch([selectedRange, statsScope], loadDashboard);");
  expect(stats).not.toContain("statsScopeLabel");
  // 用户排行榜不随视角换形态：个人与团队都显示同一份全站榜单
  expect(leaderboard).toContain("<span>总 Token</span>");
  expect(leaderboard).not.toContain("scope: StatsScope;");
  expect(leaderboard).not.toContain("我的名次");
  expect(leaderboard).not.toContain("identityId");
});
