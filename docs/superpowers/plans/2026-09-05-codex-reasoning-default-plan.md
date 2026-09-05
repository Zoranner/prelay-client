# Codex 模型推理默认值统一实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 Codex 与 ChatGPT 的推理档位、模型目录默认值和用户显式覆盖在前端及 Tauri 保存链路中保持一致。

**Architecture:** 服务端目录继续提供模型档位与默认值。客户端新增纯推理适配工具，草稿使用空覆盖表示目录默认；表单从当前目录模型生成选项，Tauri 保存入口对 Prelay 覆盖值执行最终校验。

**Tech Stack:** Nuxt 4、Vue 3、Bun、Tauri 2、Rust、Serde、现有 `CatalogLanguageModelResponse` DTO。

**Spec:** `docs/superpowers/specs/2026-09-05-codex-reasoning-default-design.md`

## Global Constraints

- 服务端目录是 `reasoning_efforts` 与 `default_reasoning_effort` 的唯一事实源。
- 空 `reasoningEffort` 表示使用模型目录默认值，不得写入 `model_reasoning_effort`。
- Prelay 保存必须同时经过前端和 Tauri 原生层校验；非法值不得写入任何 Codex 文件。
- 自定义连接不依赖服务端目录，保留现有通用档位选项。
- Rust 修改后执行 `cargo fmt --all`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-targets --all-features`。
- 前端修改后执行 `bun test`、`bun run typecheck`。

### Task 1: 建立推理档位纯逻辑适配器

**Files:**
- Create: `app/utils/modelReasoning.ts`
- Test: `tests/model-reasoning.test.ts`

**Interfaces:**
- `reasoningEffortOptions(model: CatalogLanguageModelResponse | undefined, custom: boolean): Array<{ value: string; label: string; description?: string }>`
- `normalizeReasoningEffort(value: string, model: CatalogLanguageModelResponse | undefined): string`

- [ ] **Step 1: Write the failing tests**

覆盖 GPT 六档、DeepSeek 三档、无档位模型、自定义连接和非法覆盖归一化；期望值使用手写字面量，不调用待测函数生成期望。

- [ ] **Step 2: Run the focused tests and confirm failure**

Run: `bun test ./tests/model-reasoning.test.ts`
Expected: FAIL because `modelReasoning.ts` does not exist.

- [ ] **Step 3: Implement the adapter**

按 `reasoning_efforts` 原顺序生成选项，空值首项显示目录默认档位；无档位模型返回空选项；非法非空值归一化为空字符串。

- [ ] **Step 4: Run focused tests and confirm pass**

Run: `bun test ./tests/model-reasoning.test.ts`
Expected: PASS.

### Task 2: 让草稿和保存请求表达“目录默认”

**Files:**
- Modify: `app/utils/agentSettings.ts`
- Test: `tests/model-reasoning.test.ts`

**Interfaces:**
- `createAgentConfiguration()` 返回的 Codex/ChatGPT 草稿以空字符串表示无覆盖。
- `codexSettingsPayload()` 在覆盖为空时省略 `reasoningEffort`，非空时保留原值。

- [ ] **Step 1: Add failing payload assertions**

断言新建草稿的 `reasoningEffort` 为空；空覆盖 payload 不包含 `reasoningEffort`；显式 `max` payload 保留 `reasoningEffort: "max"`。

- [ ] **Step 2: Run focused tests and confirm failure**

Run: `bun test ./tests/model-reasoning.test.ts`
Expected: FAIL because drafts currently default to `high` and payload always serializes it.

- [ ] **Step 3: Implement state and payload changes**

将默认草稿覆盖改为空，并在 payload 构造时删除空覆盖字段，不改变其他设置字段。

- [ ] **Step 4: Run focused tests and confirm pass**

Run: `bun test ./tests/model-reasoning.test.ts`
Expected: PASS.

### Task 3: 接入目录模型的表单交互与前端校验

**Files:**
- Modify: `app/composables/useAgentSettings.ts`
- Modify: `app/components/agents/CodexSettingsForm.vue`
- Modify: `app/components/agents/ChatGptSettingsForm.vue`
- Modify: `tests/model-catalog.test.ts`
- Test: `tests/agent-settings-reasoning.test.ts`

**Interfaces:**
- 设置表单接收当前 `CatalogLanguageModelResponse | undefined` 并消费动态档位选项。
- Prelay 保存校验拒绝不属于当前模型档位的非空覆盖。

- [ ] **Step 1: Write failing interaction tests**

覆盖模型切换后保留仍受支持的覆盖、清除不受支持的覆盖、无档位模型禁用选择、目录默认项为空值，以及前端保存拒绝非法覆盖。

- [ ] **Step 2: Run focused tests and confirm failure**

Run: `bun test ./tests/agent-settings-reasoning.test.ts ./tests/model-catalog.test.ts`
Expected: FAIL because options are hardcoded and save validation does not inspect reasoning coverage.

- [ ] **Step 3: Implement the interaction flow**

由 composable 根据当前模型提供目录对象；表单监听模型变化并调用归一化逻辑；自定义连接继续使用通用档位；保存前增加覆盖校验。

- [ ] **Step 4: Run focused tests and confirm pass**

Run: `bun test ./tests/agent-settings-reasoning.test.ts ./tests/model-catalog.test.ts`
Expected: PASS.

### Task 4: 在 Tauri 原生保存入口建立最终校验

**Files:**
- Modify: `src-tauri/src/agents/settings/codex.rs`
- Modify: `src-tauri/src/agents/settings/tests.rs`
- Modify: `src-tauri/src/agents/settings/validation_tests.rs`

**Interfaces:**
- Prelay 保存在生成 `models.json` 和写入 `config.toml` 前校验 `reasoning_effort`。
- 错误保持稳定中文提示，校验失败时不写入任何配置文件。

- [ ] **Step 1: Add failing native tests**

覆盖合法 `max`、非法 `medium`、无档位模型的非空覆盖，以及空覆盖不写 `model_reasoning_effort`。

- [ ] **Step 2: Run native focused tests and confirm failure**

Run: `cargo test agents::settings::`
Expected: FAIL because native save currently accepts any reasoning string and defaults `CodexSettings` to `high`.

- [ ] **Step 3: Implement native validation and default handling**

将 `CodexSettings::default().reasoning_effort` 改为空；在 Prelay 连接保存前按当前模型目录校验非空覆盖；使用已有原子写入顺序保证失败不落盘。

- [ ] **Step 4: Run native focused tests and confirm pass**

Run: `cargo test agents::settings::`
Expected: PASS.

### Task 5: 统一结构测试并完成全量验证

**Files:**
- Modify: `tests/agents-flow.test.ts`
- Modify: `src-tauri/src/agents/settings/mod.rs`
- Create/keep: `src-tauri/src/agents/settings/validation_tests.rs`

- [ ] **Step 1: Remove stale source-shape assertions**

测试只检查新的 `codex_catalog` 适配边界和可观察配置结果，不再依赖私有实现文本。

- [ ] **Step 2: Run all quality gates**

Run: `bun test`
Run: `bun run typecheck`
Run: `cargo fmt --all`
Run: `cargo clippy --all-targets --all-features -- -D warnings`
Run: `cargo test --all-targets --all-features`
Run: `git diff --check`

- [ ] **Step 3: Verify structural and behavioral acceptance**

确认 `models.json` 在目录默认、显式覆盖和无档位三种场景下均无 `null`，且非法覆盖不会生成或修改配置文件；确认所有适用 Rust 源文件不超过 450 行。
