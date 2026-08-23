# Local Extension Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 Prelay 桌面客户端自动展示本机 Codex 与 Claude Code 的用户级插件、MCP 和 Skill 状态。

**Architecture:** Tauri 原生层只读取已知用户目录和配置文件，生成不持久化的扫描快照；Nuxt 页面在挂载时调用该 command 并按客户端和类型展示。扫描器不运行任意 Skill、MCP 或 Claude Code 的健康检查，避免启动服务和产生联网副作用。

**Tech Stack:** Rust、Tauri 2、Nuxt 4、Vue 3、Bun、`toml`、`stellar-ui`、`@lobehub/icons-static-svg`。

---

### Task 1: 原生扫描模型与配置解析

**Files:**

- Create: `src-tauri/src/agent_extensions.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/src/agent_extensions.rs`

- [ ] **Step 1: 写入失败单元测试**

在 `agent_extensions.rs` 的测试模块用 `tempfile::tempdir()` 构造用户目录，写入最小的 `.codex/config.toml`、`.claude.json`、`.claude/plugins/installed_plugins.json` 和含 `SKILL.md` 的目录。断言客户端、MCP、插件、Skill 的名称与来源正确；Codex `enabled = false` 为 `disabled`；无效 JSON 或 TOML 为 `error`；不存在的客户端不返回结果。

- [ ] **Step 2: 运行测试确认失败**

Run (from `src-tauri`): `cargo test agent_extensions`

Expected: FAIL，因为扫描模块尚未定义。

- [ ] **Step 3: 实现纯本地扫描器**

定义 `AgentClient`、`AgentExtensionKind`、`AgentExtensionStatus`、`AgentExtension` 和 `AgentExtensionsSnapshot`，全部 `serde(rename_all = "camelCase")`。实现：

```rust
pub fn scan_user_extensions(home: &Path) -> AgentExtensionsSnapshot
```

Codex 解析 `.codex/config.toml` 的 `mcp_servers`、`plugins`，并扫描 `.codex/skills`、`.agents/skills`。Claude Code 解析 `.claude.json` 的 `mcpServers`、`.claude/plugins/installed_plugins.json` 中 `scope == "user"` 的记录，并扫描 `.claude/skills`、`.agents/skills`。Skill 必须含 `SKILL.md`；结果按客户端、类型、名称和路径去重；所有解析错误统一为 `error`，不回传配置内容。

在 `Cargo.toml` 加入：

```toml
toml = "0.8"
```

- [ ] **Step 4: 运行 Rust 验证**

Run:

```text
cd src-tauri
cargo test agent_extensions
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: 全部成功。

### Task 2: 暴露只读 Tauri command

**Files:**

- Create: `src-tauri/src/commands/extensions.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands/extensions.rs`

- [ ] **Step 1: 写入 command 组装测试**

抽取接受用户目录的内部函数，断言返回 `AgentExtensionsSnapshot`，且没有 `NativeState`、管理 API 或凭据依赖。

- [ ] **Step 2: 实现 `extensions_list` command**

```rust
#[tauri::command]
pub fn extensions_list() -> Result<AgentExtensionsSnapshot, ClientError>
```

通过 Windows 用户目录取得主目录并调用扫描器；在 `commands/mod.rs` 与 `lib.rs` 注册。不得执行 `codex mcp list`、`claude mcp list` 或任意第三方脚本。

- [ ] **Step 3: 运行原生全量验证**

Run:

```text
cd src-tauri
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Expected: 全部成功。

### Task 3: 前端命令、类型与扩展页面

**Files:**

- Create: `app/composables/useLocalCommand.ts`
- Create: `app/components/extensions/ExtensionList.vue`
- Create: `app/pages/extensions.vue`
- Modify: `app/stores/relay.ts`
- Modify: `app/components/workbench/WorkbenchShell.vue`
- Modify: `package.json`
- Modify: `bun.lock`
- Test: `tests/extensions-flow.test.ts`

- [ ] **Step 1: 写入失败页面测试**

创建 `tests/extensions-flow.test.ts`，断言导航含“扩展”和 `/extensions`；页面在 `onMounted` 调用 `extensions_list`；存在刷新按钮；列表包含 MCP、Skill、插件和启用、禁用、错误；客户端图标使用本地 Lobe 资源而非 `http` URL。

- [ ] **Step 2: 运行测试确认失败**

Run: `bun test tests/extensions-flow.test.ts`

Expected: FAIL，因为页面和导航尚不存在。

- [ ] **Step 3: 实现页面与展示组件**

在 `relay.ts` 定义 camelCase 扫描快照类型。新增 `useLocalCommand`，只封装本地 `invoke`、pending 和错误，不触发管理服务通知。`extensions.vue` 在挂载时加载、刷新时替换快照、未检测到客户端时不渲染其区域。`ExtensionList.vue` 使用现有 `stellar-ui` 表格和标签按类型展示名称、版本、来源路径和状态。

执行：

```text
bun add @lobehub/icons-static-svg
```

Codex 和 Claude Code 使用随包导入的静态 SVG；类型使用 `ph` 图标。将“扩展”加入 `WorkbenchShell.vue` 导航。

- [ ] **Step 4: 运行前端验证**

Run:

```text
bun test tests/extensions-flow.test.ts
bun test
bun run typecheck
```

Expected: 全部成功。

### Task 4: 完整边界核验与提交

**Files:**

- Modify: `docs/superpowers/specs/2026-08-23-local-extension-inventory-design.md`
- Test: `tests/extensions-flow.test.ts`

- [ ] **Step 1: 复核副作用边界**

确认扫描器不含网络客户端、`Command::new` 或配置写入；确认插件缓存不会在没有启用登记时显示为启用；确认一类扫描失败不会阻断同一客户端的其他结果。

- [ ] **Step 2: 运行完整验证**

Run:

```text
bun test
bun run typecheck
cd src-tauri
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
git diff --check
```

Expected: 全部成功。

- [ ] **Step 3: 提交完整功能**

Run:

```text
git add app src-tauri/src src-tauri/Cargo.toml package.json bun.lock tests docs/superpowers
git commit -m "增加本地扩展识别"
```

Expected: 单一提交只包含本功能的扫描、页面、测试、依赖与文档。
