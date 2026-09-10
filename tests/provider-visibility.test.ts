import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { identityAvatarSrc } from "../app/utils/identityAvatar";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");
const scopeSource = read(
  "../app/components/providers/ProviderVisibilityScope.vue",
);
const listSource = read("../app/components/providers/ProviderList.vue");
const avatarSource = read("../app/utils/identityAvatar.ts");

test("可见范围三态复用组件库头像组的胶囊形态", () => {
  expect(scopeSource).toContain("maxScopeAvatars = 3");
  expect(scopeSource).toContain("selected_identity_ids");
  expect(scopeSource).toContain("import { AvatarGroup");
  expect(scopeSource).toContain('icon: "ph:users-three"');
  expect(scopeSource).toContain('"全部"');
  expect(scopeSource).toContain('"私有"');
  expect(scopeSource).toContain('variant="capsule"');
  expect(scopeSource).toContain(':max="maxScopeAvatars"');
  expect(scopeSource).toContain(':aria-label="cell.title"');
  expect(scopeSource).toContain("identityAvatarSrc(entry.identity_id)");
  expect(scopeSource).not.toContain("scope-pill");
  expect(scopeSource).not.toContain("scope-chip");
  expect(listSource).toContain("ProviderVisibilityScope");
  expect(listSource).toContain("identityAvatarSrc(row.owner_identity_id)");
});

test("头像按身份缓存，同一身份只生成一次", () => {
  const first = identityAvatarSrc("identity-a");

  expect(first).toBe(identityAvatarSrc("identity-a"));
  expect(first).not.toBe(identityAvatarSrc("identity-b"));
  expect(first.startsWith("data:image/svg+xml")).toBe(true);
  expect(avatarSource).toContain("new Map<string, string>()");
});

test("可见范围单元格一次计算，不再重复遍历", () => {
  expect(scopeSource).toContain("const cell = computed");
  expect(scopeSource).toContain("const visibleEntries = computed");
  expect(scopeSource).not.toContain("scopeStack(");
});

test("状态列使用行内预计算的连通性结果", () => {
  expect(listSource).toContain("ping: pingStatus(provider.id)");
  expect(listSource).toContain("row.ping.label");
  expect(listSource).toContain("row.ping.semantic");
  expect(listSource).not.toContain("pingStatus(row.id)");
});
