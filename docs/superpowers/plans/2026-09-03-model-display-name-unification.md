# 模型显示名称统一实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让供应商、接入点、智能体配置、活动和仪表盘统一显示 ProviderCatalog 的模型显示名称，同时保持模型 ID 作为所有保存、查询、筛选和路由值。

**Architecture:** 复用协议仓库已有的完整语言/图像模型目录结构，并补充业务响应显示字段；服务端由 `ProviderCatalog` 提供唯一的 ID 到显示名称解析入口，storage 组装响应时调用；客户端集中维护目录索引，在所有页面使用目录对象的 `display_name` 作为 label。未知模型回退到 ID，不增加数据库快照字段。

**Tech Stack:** Rust、Serde、SeaORM、Axum、Vue 3/Nuxt、TypeScript、Tauri、Bun、Vitest。

**Spec:** `prelay-client/docs/superpowers/specs/2026-09-03-model-display-name-unification-design.md`

## Global Constraints

- 模型 ID 是唯一的保存、查询、筛选和路由值。
- ProviderCatalog 是显示名称的唯一权威来源，不在数据库保存显示名称快照。
- 找不到目录项时，响应中的显示名称回退为模型 ID。
- 接入点同一对外模型组的多供应商路由分别保存，客户端按模型组展示一项。
- 不增加数据库列、索引或迁移，不改变既有主键、路由唯一约束或模型 ID 规范化。
- Provider API Key、设备凭据和 Endpoint Token 不进入目录响应、模型响应或测试夹具。
- 三个仓库分别执行 Git 命令；不推送、不发布、不修改部署状态。
- Node.js 检查使用 Bun，不使用 npm、npx 或 npm lockfile。
- Rust 修改完成后执行 `cargo fmt --all` 和 `cargo clippy --all-targets --all-features -- -D warnings`。

---

### Task 1: 统一协议模型目录与业务响应

**Files:**
- Modify: `prelay-protocol/src/providers.rs`
- Modify: `prelay-protocol/src/endpoints.rs`
- Modify: `prelay-protocol/src/stats.rs`
- Modify: `prelay-protocol/src/lib.rs`
- Test: `prelay-protocol/src/providers.rs` 或现有协议测试模块

**Interfaces:**
- Consumes: 现有 `CatalogProviderResponse`、`ProviderModelResponse`、`EndpointModelResponse`、`ActivitySummary`、`ModelStatsSummary`。
- Produces: 复用已有 `CatalogLanguageModelResponse` 与 `CatalogImageGenerationModelResponse` 作为完整模型对象；业务响应新增 `display_name` 字段，wire 名称保持 snake_case；完整目录 API 使用现有 `ProviderCatalogResponse`。

- [ ] **Step 1: 写协议序列化失败测试**

  增加测试构造完整 `ProviderCatalogResponse`，断言语言和图像模型对象保留 `id`、`display_name` 及能力字段；构造 Provider、Endpoint、Activity、ModelStats 响应，断言新增字段序列化，原 ID 字段值不变。

- [ ] **Step 2: 运行协议测试确认失败**

  在 `prelay-protocol` 执行 `cargo test`，预期因新增业务字段尚未定义而编译失败。

- [ ] **Step 3: 实现协议类型**

  保持 `CatalogProviderResponse` 的供应商模型关系字段为 `Vec<String>`，不新增 `ModelReference`；给 `ProviderModelResponse` 增加 `display_name: String`，给 `EndpointModelResponse` 增加 `display_name: String`，给 `ActivitySummary` 增加 `model_requested_display_name`、`model_upstream_display_name`，给 `ModelStatsSummary` 增加 `model_requested_display_name`。在 `lib.rs` 继续导出已有完整目录类型。

- [ ] **Step 4: 运行协议测试确认通过**

  执行 `cargo fmt --all`、`cargo test`，确认序列化测试和已有协议测试通过。

- [ ] **Step 5: 提交协议仓库**

  仅暂存本任务涉及的协议文件和测试，以“补充模型显示名称协议字段”为主题提交；不要暂存其他仓库或用户既有改动。

### Task 2: 暴露完整目录并更新 Provider/Endpoint 响应

**Files:**
- Modify: `prelay-server/src/provider_catalog/mod.rs`
- Modify: `prelay-server/src/provider_catalog/response.rs`
- Modify: `prelay-server/src/routes/api/catalog.rs`
- Modify: `prelay-server/src/storage/providers.rs`
- Modify: `prelay-server/src/storage/endpoints.rs`
- Test: `prelay-server/src/provider_catalog/*` 现有测试模块、`prelay-server/src/storage` 相关测试

**Interfaces:**
- Consumes: Task 1 的新增响应字段；现有 `ProviderCatalog::language_model` 与 `image_generation_model`。
- Produces: `ProviderCatalog::language_model_response` 与 `image_generation_model_response` 作为完整目录查询；`GET /api/catalog` 返回 `ProviderCatalogResponse`；Provider/Endpoint storage 返回 display name。

- [ ] **Step 1: 写目录解析失败测试**

  为语言模型 ID、图像模型 ID、未知 ID 分别增加断言：已知项返回完整目录对象和 `display_name`，未知项在业务响应中回退 `display_name == id`；增加 `/api/catalog` 返回完整模型能力字段的断言。

- [ ] **Step 2: 运行服务端定向测试确认失败**

  在 `prelay-server` 执行对应 provider catalog 测试，预期因解析入口和响应字段尚未实现而失败。

- [ ] **Step 3: 实现统一解析器**

  复用 `ProviderCatalog::language_model_response` 与 `image_generation_model_response` 返回已有完整目录对象，在 `routes/api/catalog.rs` 增加 `GET /api/catalog`；未知模型由业务响应映射为 ID 回退。不要读取数据库或写入快照，ProviderCatalog 的供应商模型关系仍序列化为字符串 ID。

- [ ] **Step 4: 更新 Provider 与 Endpoint storage**

  修改 `provider_model_response` 与 `endpoint_model_response` 的调用链，接收 `&ProviderCatalog` 或在上层组装时补充解析结果；`model_name`、`upstream_model` 原样保留，新增 `display_name` 只由对外 ID 解析。保持现有 `(model_name, provider_id, upstream_model)` 唯一逻辑。

- [ ] **Step 5: 运行服务端验证**

  执行 `cargo fmt --all`、provider catalog/storage 定向测试和 `cargo clippy --all-targets --all-features -- -D warnings`；确认既有路由唯一性测试仍通过。

- [ ] **Step 6: 提交服务端目录与响应改动**

  仅在 `prelay-server` 暂存本任务文件和测试，提交“统一供应商和接入点模型显示名称”。

### Task 3: 更新服务端 Activity 与 ModelStats 响应

**Files:**
- Modify: `prelay-server/src/storage/activities.rs`
- Modify: `prelay-server/src/storage/stats.rs`
- Modify: `prelay-server/src/routes/api/stats.rs`
- Test: `prelay-server/src/storage/stats/tests.rs`
- Test: `prelay-server/src/routes/v1/*/tests/activities.rs` 中现有活动响应断言

**Interfaces:**
- Consumes: Task 1 的 Activity/ModelStats 显示字段；Task 2 的 `ProviderCatalog::model_reference`。
- Produces: 活动读取和模型统计读取均返回 ID 与 display name；SQL 聚合、排序和筛选仍按模型 ID。

- [ ] **Step 1: 写响应显示名称失败测试**

  在现有活动和统计测试夹具中使用目录内模型与未知模型，断言 `model_requested_display_name`、`model_upstream_display_name` 和统计显示字段分别为目录名或 ID 回退；同时断言同一模型 ID 的统计仍只有一条聚合行。

- [ ] **Step 2: 运行测试确认失败**

  执行 `cargo test storage::stats` 及活动相关测试，预期因新增字段未组装而失败。

- [ ] **Step 3: 实现 Activity 映射**

  在 `list_activities` 的 storage 映射阶段，对两个可选模型 ID 分别调用目录解析器；ID 为 `None` 时显示字段保持 `None`。持久化写入结构和日志记录函数不变。

- [ ] **Step 4: 实现 ModelStats 映射**

  在 SQL 按 `model_requested` 聚合完成后为每行补充 display name；不要把 display name 放入 `GROUP BY`、排序键或筛选条件。

- [ ] **Step 5: 运行完整服务端检查**

  执行 `cargo fmt --all`、相关测试、完整 `cargo test` 和 `cargo clippy --all-targets --all-features -- -D warnings`；若已有环境锁或无关失败，记录具体命令和边界，不改写结论。

- [ ] **Step 6: 提交服务端统计与活动改动**

  在 `prelay-server` 仅提交本任务文件和测试，提交主题为“补充活动和模型统计显示名称”。

### Task 4: 同步客户端类型并集中归一化模型引用

**Files:**
- Modify: `prelay-client/app/stores/relay.ts`
- Create: `prelay-client/app/utils/modelCatalog.ts`
- Modify: `prelay-client/app/composables/useRelayCommand.ts`
- Modify: `prelay-client/app/app.vue`
- Modify: `prelay-client/src-tauri/src/commands/providers.rs`
- Modify: `prelay-client/src-tauri/src/app/mod.rs`
- Modify: `prelay-client/app/utils/providerTemplates.ts`
- Modify: `prelay-client/app/utils/endpointModels.ts`
- Test: `prelay-client/tests/endpoint-models.test.ts`
- Test: `prelay-client/tests/provider-flow.test.ts`

**Interfaces:**
- Consumes: Task 1/2 的完整 `ProviderCatalogResponse`；旧服务端的字符串目录模型数组。
- Produces: 直接复用 `CatalogLanguageModelResponse`/`CatalogImageGenerationModelResponse` 的客户端类型；`modelCatalogLabel(id)`、`modelCatalogEntry(id)` 等集中查询函数；Endpoint 模型组以对外模型 ID 作为稳定分组键并保留目录对象。

- [ ] **Step 1: 写归一化失败测试**

  增加 Vitest 用例覆盖完整目录对象查询、旧字符串供应商模型 ID 关联、未知 ID 回退，以及同一对外模型 ID 的多供应商映射合并。

- [ ] **Step 2: 运行定向测试确认失败**

  在 `prelay-client` 执行 `bun test tests/endpoint-models.test.ts tests/provider-flow.test.ts`，预期新断言失败。

- [ ] **Step 3: 更新 TypeScript 类型和工具**

  在 `relay.ts` 同步完整目录类型和 Provider/Endpoint/Activity/ModelStats 字段；新增 `modelCatalog.ts` 保存目录索引并提供按 ID 查询对象/显示名称的函数。修改 `groupEndpointModels` 以 `model_name`（当前对外模型 ID）作为稳定分组键，并在组对象中保留目录对象供渲染，提交值仍为模型 ID。

- [ ] **Step 4: 增加目录命令并在页面框架加载**

  在 Tauri `catalog_models_get` 命令中请求 `/api/catalog` 并返回 `ProviderCatalogResponse`，注册到 `app/mod.rs` 和 `useRelayCommand`；在应用框架初始化时加载一次并写入 `modelCatalog.ts` 索引，加载失败时保留空索引并允许 ID 回退。

- [ ] **Step 5: 更新模板转换**

  `providerTemplates` 将供应商模型 ID 关联到完整目录对象生成选项；所有表单 payload 继续输出字符串模型 ID。

- [ ] **Step 6: 运行客户端定向检查**

  执行 `bun test tests/endpoint-models.test.ts tests/provider-flow.test.ts` 和仓库已有类型/格式脚本，确认兼容旧字符串响应。

### Task 5: 更新供应商、接入点和智能体配置消费

**Files:**
- Modify: `prelay-client/app/composables/useProviderForm.ts`
- Modify: `prelay-client/app/components/providers/ProviderForm.vue`
- Modify: `prelay-client/app/components/providers/ProviderList.vue`
- Modify: `prelay-client/app/components/endpoints/EndpointForm.vue`
- Modify: `prelay-client/app/components/endpoints/EndpointList.vue`
- Modify: `prelay-client/app/composables/useAgentSettings.ts`
- Modify: `prelay-client/app/utils/agentSettings.ts`
- Modify: `prelay-client/app/components/agents/CodexSettingsForm.vue`
- Modify: `prelay-client/app/components/agents/ChatGptSettingsForm.vue`
- Modify: `prelay-client/app/components/agents/OpenCodeSettingsForm.vue`
- Modify: `prelay-client/src-tauri/src/agents/settings/codex.rs`
- Modify: `prelay-client/src-tauri/src/agents/settings/mod.rs`
- Test: `prelay-client/tests/provider-flow.test.ts`
- Test: `prelay-client/tests/endpoint-flow.test.ts`
- Test: `prelay-client/tests/agents-flow.test.ts`

**Interfaces:**
- Consumes: Task 4 的目录索引；服务端 Provider/Endpoint display name 字段。
- Produces: 所有选择器 label 使用完整目录对象的 `display_name`；保存模型 ID；Codex `slug` 使用 ID、`display_name` 与能力字段使用目录对象。

- [ ] **Step 1: 写页面行为失败测试**

  扩展供应商、接入点和智能体流程测试，断言渲染文字为 display name、提交 body/model 字段为 ID；增加多供应商同组只出现一个对外模型项的断言。

- [ ] **Step 2: 运行流程测试确认失败**

  执行 `bun test tests/provider-flow.test.ts tests/endpoint-flow.test.ts tests/agents-flow.test.ts`，记录新断言失败位置。

- [ ] **Step 3: 改造供应商与接入点表单**

  供应商表单保留 `string[]` 草稿值并从完整目录对象生成 label；接入点列表/表单展示目录对象的 `display_name`，提交继续发送 `provider_id`、`upstream_model` 和模型 ID。不得恢复可自定义对外模型名。

- [ ] **Step 4: 改造智能体配置**

  `useAgentSettings` 生成带显示标签的模型选项，connection 中 `modelName`/`upstreamModel` 继续使用 ID，并把对应 `CatalogLanguageModelResponse` 随模型项传给本地命令。扩展 `CodexEndpointModel` 的精确字段为 `catalog_model: Option<CatalogLanguageModelResponse>`；Tauri Codex 写目录时以该目录对象为配置档案来源，`slug` 仍写模型 ID；未知模型使用现有模板并以 ID 作为显示名称。

- [ ] **Step 5: 运行客户端检查**

  执行上述定向测试、`bun run typecheck` 和格式检查；确认现有 `@stellar/ui` 版本断言失败与本任务无关时单独记录。

### Task 6: 更新活动页和仪表盘模型展示

**Files:**
- Modify: `prelay-client/app/components/activity/RequestTable.vue`
- Modify: `prelay-client/app/components/dashboard/StatsBreakdownTable.vue`
- Modify: `prelay-client/app/pages/index.vue`
- Modify: `prelay-client/app/pages/stats.vue`
- Test: `prelay-client/tests/stats-flow.test.ts`

**Interfaces:**
- Consumes: Task 4 的 Activity/ModelStats 类型和 display name 字段。
- Produces: 活动模型列、仪表盘模型统计和趋势相关模型标签显示 display name；查询参数、筛选值和统计键保持 ID。

- [ ] **Step 1: 写展示失败测试**

  在活动和统计流程测试 fixtures 中同时提供 ID/display name，断言用户看到 display name；缺失 display name 时断言显示 ID，不改变请求参数。

- [ ] **Step 2: 运行测试确认失败**

  执行 `bun test tests/stats-flow.test.ts`，预期新显示断言失败。

- [ ] **Step 3: 更新活动表**

  请求模型列使用 `model_requested_display_name`，供应商/上游列使用 `model_upstream_display_name`，仅在字段缺失时回退相应 ID；活动接口 limit/status 逻辑不变。

- [ ] **Step 4: 更新仪表盘统计**

  模型统计行和图表标签使用 `model_requested_display_name`，内部 key 和排序仍使用 `model_requested` 及原数值；不把 display name 作为查询筛选参数。

- [ ] **Step 5: 运行客户端回归检查**

  执行定向测试、完整 `bun test`、类型检查和格式检查；将既有失败与本次新增失败分开记录。

### Task 7: 跨仓契约回归与收口

**Files:**
- Modify: `prelay-client/crates/protocol`（更新 submodule 指针）
- Modify: `prelay-client` 中受协议变更影响的 Tauri command 类型文件
- Test: `prelay-server/tests` 相关管理 API 测试
- Test: `prelay-client/src-tauri/tests/command_registration.rs`、管理命令契约测试

**Interfaces:**
- Consumes: Tasks 1-6 已提交的协议、服务端和客户端改动。
- Produces: 三仓库使用同一协议版本，管理 API 到 Tauri 到 Vue 的 JSON 字段完整贯通。

- [ ] **Step 1: 更新并检查协议 submodule**

  在 `prelay-client` 更新 `crates/protocol` 指向 Task 1 的协议提交；确认 `prelay-server` 的协议依赖同样指向该提交，分别检查 submodule status。

- [ ] **Step 2: 运行跨边界静态测试**

  执行服务端管理 API 测试、客户端 Tauri command contract 测试和前端 API boundary 测试，断言新增字段没有被 command 层丢弃。

- [ ] **Step 3: 复查完整 diff 与敏感信息边界**

  分别执行 `git diff --check`、检查 staged/unstaged 差异，确认没有数据库迁移、模型显示名称写入凭据、API Key 或 Token 的日志/fixture 泄露，也没有混入用户既有改动。

- [ ] **Step 4: 运行最终验证**

  服务端执行 `cargo fmt --all`、`cargo clippy --all-targets --all-features -- -D warnings` 和可运行的测试；客户端执行 Bun 类型检查、格式检查和测试。最终报告必须区分通过项与环境/既有失败。

- [ ] **Step 5: 分仓提交并停止**

  按逻辑单元分别提交协议、服务端、客户端改动；不执行 push、tag、release 或部署。
