# 模型目录与智能体配置统一重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让服务端 `ProviderCatalog` 成为供应商、接入点和 Codex `models.json` 的唯一模型事实源，移除客户端模板回退与可缺失目录副本。

**Architecture:** 共享协议只表达目录模型 ID 和完整目录对象；服务端在 Provider/Endpoint 写入事务前校验所有模型 ID 及供应商关系；客户端启动时加载一次完整目录并建立索引，Prelay 智能体保存负载携带完整语言模型对象，自定义连接完全绕开目录文件生成。

**Tech Stack:** Rust、Axum、SeaORM、Tauri 2、Nuxt 4、Vue 3、TypeScript、Bun、serde/serde_json。

**Spec:** `docs/superpowers/specs/2026-09-04-model-catalog-save-redesign-design.md`

## Global Constraints

- `prelay-server/config/catalog` 是模型、显示名称、能力、协议和供应商支持关系的唯一事实源。
- 数据库只保存模型 ID、供应商 ID 和路由关系，不保存显示名称或能力快照。
- Provider 与 Endpoint 不接受目录外模型或自定义对外模型名；同一对外模型 ID 可有多个供应商候选，不同模型 ID 不得混组。
- Prelay 智能体必须携带完整 `CatalogLanguageModelResponse`；自定义连接不携带目录对象，也不生成 `models.json`。
- 不恢复上游模型发现、客户端内置供应商/模型模板或静默空目录回退。
- 不修改工作区中用户已有的 `providers.toml`、`TokenUsageTrendChart.vue`、`bun.lock`、`package.json`、`tests/app-shell.test.ts` 改动；不推送、不发布、不改变部署状态。

---

### Task 1: 收紧共享 Endpoint 模型契约

**Files:**
- Modify: `prelay-protocol/src/endpoints.rs`
- Modify: `prelay-protocol/src/lib.rs`（保持导出一致）
- Modify: `prelay-protocol/tests/management_dto.rs`
- Modify: `prelay-server/src/storage/endpoints.rs`
- Modify: `prelay-server/tests/management/endpoints.rs`
- Modify: `prelay-client/app/stores/relay.ts`
- Modify: `prelay-client/app/composables/useAgentSettings.ts`

**Interfaces:**
- `EndpointModelInput` 仅包含 `provider_id: String` 和 `upstream_model: String`；删除可选 `model_name`。
- `EndpointModelResponse.model_name` 保持服务端响应字段，但值始终等于目录模型 ID；`display_name` 由目录投影。
- 前端 `EndpointModel` 保留响应字段用于展示，但创建/更新负载不再发送 `model_name`。

- [ ] **Step 1: 写失败测试**

在 `prelay-protocol/tests/management_dto.rs` 增加反序列化断言：包含 `modelName` 的 Endpoint 输入被拒绝；不包含 `modelName` 的输入可 round-trip。更新服务端端点测试 payload，断言目录外或自定义对外名称不再被接受。

- [ ] **Step 2: 运行测试确认失败**

运行：`cargo test --manifest-path prelay-protocol/Cargo.toml --all-targets --all-features management_dto`。
预期：现有可选字段仍会被接受，新增拒绝断言失败。

- [ ] **Step 3: 实现最小契约变更**

从 `EndpointModelInput` 删除 `model_name` 及默认名称辅助逻辑；服务端 `normalize_models` 将 `model_name` 直接设为 `upstream_model`；客户端接入点保存请求只提交 `provider_id` 与 `upstream_model`。

- [ ] **Step 4: 运行协议与受影响测试**

运行：`cargo fmt --all --manifest-path prelay-protocol/Cargo.toml`、`cargo clippy --manifest-path prelay-protocol/Cargo.toml --all-targets --all-features -- -D warnings`、`cargo test --manifest-path prelay-protocol/Cargo.toml --all-targets --all-features`，以及 `cargo test --manifest-path prelay-server/Cargo.toml --test management endpoints`。
预期：全部通过，服务端测试请求不再发送 `model_name`。

- [ ] **Step 5: 提交**

在 `prelay-protocol` 提交 `收紧接入点模型输入契约`；在父仓仅在客户端和服务端适配完成后更新 submodule 指针。

### Task 2: 服务端目录关系校验

**Files:**
- Modify: `prelay-server/src/storage/providers.rs`
- Modify: `prelay-server/src/storage/endpoints.rs`
- Modify: `prelay-server/src/storage/mod.rs`（复用既有稳定校验错误）
- Modify: `prelay-server/tests/management/providers.rs`
- Modify: `prelay-server/tests/management/endpoints.rs`
- Modify: `prelay-server/tests/management/provider_operations.rs`

**Interfaces:**
- `create_with_catalog`/`update_with_catalog` 在事务写入前调用目录校验；无目录的旧内部 helper 不绕过管理路由。
- Provider 校验使用 `ProviderCatalog::provider`, `provider_supports_language_model` 和 `provider_supports_image_generation_model`。
- Endpoint 校验要求每个 `upstream_model` 属于所选 Provider 的目录模型集合，且所有候选属于同一模型 ID 组。

- [ ] **Step 1: 写失败测试**

增加管理 API 测试：未知 `provider_type`、Provider 目录外模型、Endpoint 使用目录外模型、Endpoint 使用不支持该模型的 Provider 均返回 `validation_failed`；同一模型 ID 的多个 Provider 候选保存成功；不同模型 ID 不能在同一映射中伪装成一个对外组。

- [ ] **Step 2: 运行测试确认失败**

运行：`cargo test --manifest-path prelay-server/Cargo.toml --test management providers endpoints -- --nocapture`。
预期：当前实现会接受至少一个目录外模型或不支持关系，新增断言失败。

- [ ] **Step 3: 实现目录校验**

在 Provider 创建/更新的事务内校验 `provider_type` 存在且每个模型 ID 被该目录 Provider 支持；在 Endpoint 规范化后校验 Provider 属于当前 identity、模型 ID 在 Provider 目录关系中存在，并将 `model_name` 固定为 `upstream_model`。返回 `StorageError::ValidationFailed`，不写入部分状态。

- [ ] **Step 4: 运行服务端完整验证**

运行：`cargo fmt --all`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-targets --all-features`、`git diff --check`（均在 `prelay-server`）。

- [ ] **Step 5: 提交**

在 `prelay-server` 提交 `按模型目录校验供应商和接入点关系`，不包含用户已有的 `config/catalog/providers.toml` 改动。

### Task 3: 客户端统一目录索引和保存前置条件

**Files:**
- Modify: `prelay-client/app/utils/modelCatalog.ts`
- Modify: `prelay-client/app/app.vue`
- Modify: `prelay-client/app/composables/useAgentSettings.ts`
- Modify: `prelay-client/app/utils/endpointModels.ts`
- Modify: `prelay-client/app/composables/useProviderForm.ts`
- Modify: `prelay-client/app/components/providers/ProviderForm.vue`
- Modify: `prelay-client/app/pages/providers.vue`
- Modify: `prelay-client/app/components/endpoints/EndpointForm.vue`
- Modify: `prelay-client/app/pages/endpoints.vue`
- Test: `prelay-client/tests/model-catalog.test.ts`（按现有测试组织调整）

**Interfaces:**
- `useModelCatalog()` 暴露 `status: "idle" | "loading" | "ready" | "error"`、完整 `catalog`、按 ID 查询及按 Provider 过滤函数。
- `useAgentSettings` 从统一索引把接入点模型 ID 映射为完整 `CatalogLanguageModelResponse`；Prelay 保存目录未 `ready` 时返回明确校验错误。
- Provider/Endpoint 页面只使用目录 ID 选择器，不提供模型发现、新增自定义模型或自定义对外名称入口。

- [ ] **Step 1: 写失败测试**

增加 Bun 测试：目录加载状态从 `loading` 到 `ready` 时索引可查；目录请求失败状态为 `error`；Prelay 保存请求在非 `ready` 状态下不调用 Tauri save；自定义连接在目录 `error` 时仍可保存；模型选择器只返回目录支持的语言/图像模型。

- [ ] **Step 2: 运行测试确认失败**

运行：`bun test tests/model-catalog.test.ts`。
预期：当前 composable 没有显式状态，保存仍会继续，新增断言失败。

- [ ] **Step 3: 实现单一目录来源**

将 `setModelCatalog` 改为带状态的原子更新；`app.vue` 在请求开始、成功、失败时设置状态并清理旧目录；所有模型选项从该索引派生，移除空索引静默回退及上游发现路径。

- [ ] **Step 4: 运行客户端定向验证**

运行：`bun test tests/model-catalog.test.ts tests/agents-flow.test.ts`、`bun run typecheck`、`bun run generate`。
预期：定向测试、类型检查和静态生成通过。

- [ ] **Step 5: 提交**

在 `prelay-client` 提交 `统一客户端模型目录状态与选择器`，不暂存用户已有 UI、依赖和壳测试改动。

### Task 4: 重写 Codex `models.json` 生成链路

**Files:**
- Modify: `prelay-client/src-tauri/src/agents/settings/mod.rs`
- Modify: `prelay-client/src-tauri/src/agents/settings/codex.rs`
- Modify: `prelay-client/src-tauri/src/agents/settings/tests.rs`
- Delete: `prelay-client/src-tauri/src/agents/settings/deepseek_models.json`
- Modify: `prelay-client/tests/agents-flow.test.ts`

**Interfaces:**
- `CodexConnection::Prelay.models: Vec<CatalogLanguageModelResponse>`；不再存在 `CodexEndpointModel` 或 `catalog_model: Option<_>`。
- `write_prelay_model_catalog(home, models: &[CatalogLanguageModelResponse])` 逐项序列化完整目录对象，设置 `slug = id`，返回写入路径。
- `validate_prelay_default_model(model, models)` 按目录 ID 校验默认模型。

- [ ] **Step 1: 写失败测试**

重写 Tauri 设置测试，构造包含 `display_name`、`reasoning_efforts`、`context_window` 和自定义 `base_instructions` 的完整目录对象，断言生成文件保留这些值且不包含 DeepSeek 模板字段；断言目录对象缺失时无法反序列化；断言自定义连接不会创建新的 `models.json` 或设置 `model_catalog_json`；断言写入失败时 `config.toml` 不落半成品。

- [ ] **Step 2: 运行测试确认失败**

运行：`cargo test --manifest-path prelay-client/src-tauri/Cargo.toml agents::settings::tests -- --nocapture`。
预期：当前模型类型仍是 endpoint alias，模板字段断言失败或缺少目录对象不会失败。

- [ ] **Step 3: 实现直接序列化**

删除 `include_str!("deepseek_models.json")`、模板查找、首项 fallback 和人为覆盖显示名逻辑；先完成目录文件原子写入，再更新配置中的 `model_catalog_json`；Custom 分支只更新 provider URL/token，不创建目录文件。

- [ ] **Step 4: 运行 Tauri 验证**

运行：`cargo fmt --all`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-targets --all-features`（在 `prelay-client/src-tauri`），以及 `bun test tests/agents-flow.test.ts`。

- [ ] **Step 5: 提交**

在 `prelay-client` 提交 `按服务端目录生成 Codex 模型配置`，确认删除模板文件已包含在提交中。

### Task 5: 跨仓契约、消费者和敏感信息回归

**Files:**
- Modify: `prelay-client/crates/protocol` submodule pointer
- Modify: `prelay-server/crates/protocol` submodule pointer
- Modify: `prelay-client/tests/agents-flow.test.ts`、相关 API fixture
- Modify: `prelay-server/tests/management/provider_catalog.rs`

**Interfaces:**
- 两个父仓使用同一个 `prelay-protocol` 提交；完整 `ProviderCatalogResponse` JSON 在 Rust 与 TypeScript 中字段一致。
- Provider/Endpoint/Activity/ModelStats 的显示字段继续来自目录投影，数据库 fixture 只保存 ID。

- [ ] **Step 1: 写跨仓回归断言**

断言 `/api/catalog` 返回完整语言模型对象；客户端把同一对象传入 Tauri；生成的 `models.json` 的 `slug`、`display_name`、能力字段与 `/api/catalog` 输入一致；扫描测试输出和持久化文件不出现 API Key、Endpoint Token 或 device credential。

- [ ] **Step 2: 运行失败测试并定位契约差异**

运行：`bun test` 与 `cargo test --all-targets --all-features`（分别在客户端 Tauri、服务端、协议仓），记录只属于用户既有 `@stellar/ui` 版本断言的失败，不把它们误归因于本重构。

- [ ] **Step 3: 更新 submodule 并修正消费者**

在两个父仓执行 `git submodule update --init --recursive`，检查协议指针；只修正由 `EndpointModelInput` 和目录状态变更导致的调用方，不复制协议类型。

- [ ] **Step 4: 完成最终验证**

客户端执行 `bun test`、`bun run typecheck`、`bun run generate`；Tauri 与服务端执行格式化、Clippy、全量测试和 `git diff --check`；人工检查 `models.json` 仅含服务端目录对象，且自定义连接没有新建该文件。

- [ ] **Step 5: 按责任仓拆分提交并收口**

确认每个仓的 staged diff 只包含本计划变更和对应 submodule 指针，采用仓库既有中文提交风格分别提交；不暂存、不提交用户已有改动，不推送远端。
