# 智能体扩展库实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在智能体页提供固定 Gitea 组织的扩展发现、README 预览和规则/Skill 本机安装。

**Architecture:** 原生层拥有 Gitea 查询、manifest 校验、固定 commit 下载、本机原子写入与安装记录；Nuxt 只通过 Tauri command 维护目录和弹窗状态。扩展库复用现有智能体页的左侧列表、分段控制和表格，详情使用抽屉，安装使用单层模态框。

**Tech Stack:** Tauri 2、Rust、reqwest、serde、Nuxt 4、Vue、Bun、@stellar/ui。

**Spec:** `docs/superpowers/specs/2026-08-27-agent-extensions-design.md`

## 全局约束

- 扩展源固定为 `https://git.kimo.ink/agents`，不经过 Prelay 服务端。
- 不执行远程脚本，不存储 token、密码、OAuth 凭据或其他用户秘密。
- 已安装的 Codex CLI 与 ChatGPT 分别显示但共享落点只写入一次。
- 首期实际安装 `rule` 与 `skill`；`plugin` 与 `mcp` 只发现和预览。
- 所有远程文件必须按解析出的 commit SHA 下载。
- Rust 修改后执行 `cargo fmt --all`、`cargo clippy --all-targets --all-features -- -D warnings` 和可运行的最大范围测试；Node 命令只使用 Bun。

---

### Task: 原生扩展目录模型与发现

**Files:**
- Create: `src-tauri/src/extensions.rs`
- Create: `src-tauri/src/commands/extensions.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/agents.rs`
- Test: `src-tauri/src/extensions.rs`

**Interfaces:**
- Produces `ExtensionPackage`, `ExtensionKind`, `ExtensionCatalogSnapshot` and `extensions_list()` Tauri command.
- Consumes existing `AgentClient` detection for installation-target availability.

- [ ] **Step: 写出发现与 manifest 过滤的失败测试**

```rust
#[test]
fn catalog_ignores_repositories_with_invalid_manifests() {
    let packages = parse_catalog_entries(repositories, manifests);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].kind, ExtensionKind::Skill);
}
```

- [ ] **Step: 运行失败测试并确认失败来自缺少目录模型**

Run: `cargo test extensions::tests::catalog_ignores_repositories_with_invalid_manifests`

- [ ] **Step: 实现固定组织分页查询、commit SHA 解析和 manifest 校验**

```rust
pub async fn list_extensions() -> Result<ExtensionCatalogSnapshot, String> {
    ExtensionCatalogClient::new(EXTENSION_API_URL, EXTENSION_ORGANIZATION)
        .list_packages()
        .await
}
```

- [ ] **Step: 重运行 Rust 目标测试并确认通过**

Run: `cargo test extensions::tests::catalog_ignores_repositories_with_invalid_manifests`

- [ ] **Step: 提交原生目录发现模块**

```text
git add src-tauri/src/extensions.rs src-tauri/src/commands/extensions.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/agents.rs
git commit -m "新增智能体扩展目录发现"
```

### Task: 原生规则与 Skill 安装

**Files:**
- Modify: `src-tauri/src/extensions.rs`
- Modify: `src-tauri/src/commands/extensions.rs`
- Test: `src-tauri/src/extensions.rs`

**Interfaces:**
- Consumes `ExtensionPackage { repository, commit_sha, kind }` and `ExtensionInstallRequest { clients }`.
- Produces `extensions_install()` Tauri command and a preview containing copied files and merged rule targets.

- [ ] **Step: 写出规则托管段落和 Skill 目标映射的失败测试**

```rust
#[test]
fn merges_one_managed_rule_block_without_replacing_user_rules() {
    assert!(merge_managed_rule("# user", "development-rules", "# package").contains("# user"));
}

#[test]
fn codex_and_chatgpt_share_one_skill_target() {
    assert_eq!(skill_targets(&[AgentClient::CodexCli, AgentClient::ChatGpt]).len(), 1);
}
```

- [ ] **Step: 运行失败测试并确认失败来自缺少安装实现**

Run: `cargo test extensions::tests::merges_one_managed_rule_block_without_replacing_user_rules`

- [ ] **Step: 实现固定 commit 下载、预览、原子复制和规则合并**

```rust
pub fn install_extension(
    home: &Path,
    package: &ExtensionPackage,
    clients: &[AgentClient],
) -> Result<ExtensionInstallResult, String>
```

- [ ] **Step: 重运行 Rust 安装测试并确认通过**

Run: `cargo test extensions::tests`

- [ ] **Step: 提交原生安装模块**

```text
git add src-tauri/src/extensions.rs src-tauri/src/commands/extensions.rs
git commit -m "支持规则与技能包本机安装"
```

### Task: 扩展库界面与本地状态

**Files:**
- Create: `app/composables/useExtensionCatalog.ts`
- Create: `app/components/extensions/ExtensionCatalogTable.vue`
- Create: `app/components/extensions/ExtensionDetailDrawer.vue`
- Create: `app/components/extensions/ExtensionInstallModal.vue`
- Modify: `app/composables/useLocalCommand.ts`
- Modify: `app/stores/relay.ts`
- Modify: `app/pages/agents.vue`
- Test: `tests/extensions-flow.test.ts`

**Interfaces:**
- Consumes `extensions_list`, `extension_readme`, `extensions_install` through `useLocalCommand`.
- Produces the left “扩展库” entry and right `规则 / 插件 / MCP / Skill` table views.

- [ ] **Step: 写出扩展库入口、表格操作、抽屉和弹窗的失败结构测试**

```ts
test("扩展库沿用智能体工作区的分类表格与单层操作表面", () => {
  expect(page).toContain("扩展库");
  expect(page).toContain("<ExtensionCatalogTable");
  expect(page).toContain("<ExtensionDetailDrawer");
  expect(page).toContain("<ExtensionInstallModal");
});
```

- [ ] **Step: 运行失败测试并确认失败来自缺少扩展库界面**

Run: `bun test tests/extensions-flow.test.ts`

- [ ] **Step: 实现列表加载、分类表格、README 抽屉和联动安装弹窗**

```ts
const codexHostSelected = computed({
  get: () => targets.codexCli || targets.chatgpt,
  set: (value) => {
    targets.codexCli = value && detected.codexCli;
    targets.chatgpt = value && detected.chatgpt;
  },
});
```

- [ ] **Step: 重运行前端目标测试并确认通过**

Run: `bun test tests/extensions-flow.test.ts`

- [ ] **Step: 提交扩展库界面模块**

```text
git add app/composables/useExtensionCatalog.ts app/components/extensions app/composables/useLocalCommand.ts app/stores/relay.ts app/pages/agents.vue tests/extensions-flow.test.ts
git commit -m "新增智能体扩展库工作区"
```

### Task: 全量验证与提交复查

**Files:**
- Modify: files created by the preceding tasks only when verification exposes a defect.

**Interfaces:**
- Verifies all Tauri command and Nuxt interaction contracts introduced above.

- [ ] **Step: 执行格式化和完整静态检查**

Run: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `bun test`; `bun run typecheck`

- [ ] **Step: 执行 Rust 测试并记录运行中的客户端锁定边界**

Run: `cargo test --all-targets --all-features`

- [ ] **Step: 检查每个提交的范围与工作树**

Run: `git log --format="%h %s" -n 4`; `git status --short --branch`; `git diff --check`
