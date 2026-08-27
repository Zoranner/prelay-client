# Stellar UI 接入与智能体页面实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 收回组件库离线图标和样式顺序职责，简化 Prelay 的 Nuxt 接入，并把智能体页面拆为页面编排与稳定展示组件。

**Architecture:** Stellar UI 的 Nuxt 模块只负责自己的基础样式和固定内部图标；宿主负责开启 `@nuxt/icon` 本地扫描并提供业务源码。Prelay 页面继续拥有本地命令、草稿、加载、保存和退出守卫，规则编辑器与两种设置表单变为受控展示组件。

**Tech Stack:** Nuxt 4、Vue 3、TypeScript、Bun、`@nuxt/icon`、Stellar UI。

---

### Task 1: Stellar UI 模块样式契约

**Files:**

- Modify: `E:\Repositories\projects\Zoranner\workflow-suite\stellar-ui\src\module.ts`
- Modify: `E:\Repositories\projects\Zoranner\workflow-suite\stellar-ui\tests\nuxt-module.test.ts`
- Modify: `E:\Repositories\projects\Zoranner\workflow-suite\stellar-ui\package.json`

- [ ] **Step 1: 先增加失败测试**

在 `tests/nuxt-module.test.ts` 断言模块在 `modules:done` 时将 `@stellar/ui/styles` 放到 CSS 列表首位，并且仍不读取或修改 `nuxt.options.icon`。

- [ ] **Step 2: 执行失败测试**

Run: `bun test tests/nuxt-module.test.ts`

Expected: 新的样式顺序断言失败，因为模块当前只调用 `nuxt.options.css.push(...)`。

- [ ] **Step 3: 实现模块所有权**

在 `setup` 中保留 CSS 注册；注册 `modules:done` hook，将模块自己注册的 `@stellar/ui/styles` 移到 CSS 数组首位。图标 hook 保持只向 `icon:clientBundleIcons` 增加 `stellarInternalIcons`。

- [ ] **Step 4: 重新发布组件库版本**

将样式顺序契约与既有内部图标 hook 一并重新发布为 `0.1.4`；发布后清理该版本的本地 Bun 缓存，再由客户端重新解析其 tarball。

- [ ] **Step 5: 执行局部验证**

Run: `bun test tests/nuxt-module.test.ts && bun run typecheck`

Expected: 测试和类型检查通过。

### Task 2: Stellar UI Nuxt 离线接入文档

**Files:**

- Modify: `E:\Repositories\projects\Zoranner\workflow-suite\stellar-ui\README.md`
- Modify: `E:\Repositories\projects\Zoranner\workflow-suite\stellar-ui\tests\nuxt-module.test.ts`

- [ ] **Step 1: 先增加失败测试**

在模块测试中读取 README，断言 Nuxt 示例包含 `@nuxt/icon`、`provider: 'none'`、`componentName: 'NuxtIcon'`、`clientBundle.scan: true`，且示例不再手工声明 `css: ['@stellar/ui/styles']`。

- [ ] **Step 2: 执行失败测试**

Run: `bun test tests/nuxt-module.test.ts`

Expected: 文档断言失败。

- [ ] **Step 3: 更新 README**

提供 Nuxt 离线接入示例，并明确：组件库声明其固定内部图标；宿主用扫描收集普通静态业务图标；运行时拼接图标不在扫描能力内，才使用有限显式清单。说明模块自动加载组件库样式，宿主不重复导入。

- [ ] **Step 4: 执行局部验证**

Run: `bun test tests/nuxt-module.test.ts`

Expected: 测试通过。

### Task 3: Prelay 接入契约与离线 bundle

**Files:**

- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\package.json`
- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\bun.lock`
- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\nuxt.config.ts`
- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\tests\icon-runtime.test.ts`
- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\tests\legacy-web-layout.test.ts`

- [ ] **Step 1: 先增加失败测试**

将图标运行测试扩展为读取生成客户端 bundle，断言 Phosphor 的 `sliders-horizontal`、`info`、`check-circle`、`x-circle` 都存在；将布局测试改为断言 `nuxt.config.ts` 没有 `modules:done` 排序补丁。

- [ ] **Step 2: 执行失败测试**

Run: `bun test tests/icon-runtime.test.ts tests/legacy-web-layout.test.ts`

Expected: 内部反馈图标和配置补丁断言失败。

- [ ] **Step 3: 消费发布的组件库补丁版本**

在组件库补丁版本已发布到现有私有 registry 后，用 Bun 更新 `@stellar/ui` 和 lockfile；删除 Prelay 的匿名样式排序模块，仅保留 Nuxt Icon 的离线扫描配置。

- [ ] **Step 4: 验证 bundle**

Run: `bun run generate`，然后通过 `init()` 读取 `.nuxt/nuxt-icon-client-bundle.mjs`。

Expected: 业务滑杆图标与组件库通知图标同时存在，未依赖远程图标服务或全量图标集合。

### Task 4: 无框页面工作区

**Files:**

- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\app\components\shell\PanelSection.vue`
- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\tests\legacy-web-layout.test.ts`

- [ ] **Step 1: 先增加失败测试**

断言 `PanelSection` 不导入或渲染 Stellar UI 的 Card，但保留页面标题、header actions 和内容插槽。

- [ ] **Step 2: 执行失败测试**

Run: `bun test tests/legacy-web-layout.test.ts`

Expected: 现有 Card 实现导致断言失败。

- [ ] **Step 3: 实现无框结构**

移除外层 Card，保留原有全高 flex、标题分隔线、内容 padding 和滚动约束。仪表盘的 StatCard、图表 Card 和其他独立内容不修改。

- [ ] **Step 4: 执行局部验证**

Run: `bun test tests/legacy-web-layout.test.ts && bun run typecheck`

Expected: 测试和类型检查通过。

### Task 5: 智能体页面展示职责拆分

**Files:**

- Create: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\app\components\agents\AgentRulesEditor.vue`
- Create: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\app\components\agents\CodexSettingsForm.vue`
- Create: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\app\components\agents\ClaudeCodeSettingsForm.vue`
- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\app\pages\agents.vue`
- Modify: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client\tests\agents-page.test.ts`

- [ ] **Step 1: 先增加失败测试**

断言智能体页导入三个展示组件并保留“滑杆图标 + 配置”入口；分别读取新组件文件并断言规则编辑器有 Textarea/MarkdownViewer，两个表单各自只承载对应智能体字段。

- [ ] **Step 2: 执行失败测试**

Run: `bun test tests/agents-page.test.ts`

Expected: 新组件文件和页面导入断言失败。

- [ ] **Step 3: 提取规则编辑器**

让 `AgentRulesEditor.vue` 以 `v-model` 接收规则文本，并在组件内部管理编辑区与预览区的双向滚动同步。页面继续维护自动保存、脏状态、保存中状态与退出守卫。

- [ ] **Step 4: 提取设置表单**

让 Codex 和 Claude Code 表单以受控 draft、选项和可见状态作为输入。页面继续计算连接信息、调用 `agent_settings_save`、维护草稿、通知和 Drawer footer；Codex 表单在抽屉可见时从非自定义接入点切换到自定义接入点，清空自定义 URL 与 Token。

- [ ] **Step 5: 执行局部验证**

Run: `bun test tests/agents-page.test.ts tests/agents-flow.test.ts tests/workspace-exit-guard.test.ts && bun run typecheck`

Expected: 测试和类型检查通过，页面不再包含规则滚动实现或两类表单字段标记。

### Task 6: 整合验证

**Files:**

- Verify only: `E:\Repositories\projects\Zoranner\provider-relay\prelay-client`
- Verify only: `E:\Repositories\projects\Zoranner\workflow-suite\stellar-ui`

- [ ] **Step 1: 运行组件库完整检查**

Run: `bun test && bun run typecheck`

Expected: 全部组件库测试与类型检查通过。

- [ ] **Step 2: 运行客户端完整检查**

Run: `bun test && bun run typecheck && bun run generate`

Expected: 全部客户端测试、类型检查和静态生成通过。

- [ ] **Step 3: 核验生成离线图标 bundle**

Run: 读取 `.nuxt/nuxt-icon-client-bundle.mjs` 的 `init()` 输出。

Expected: bundle 含业务滑杆图标和组件库反馈图标，且客户端配置仍为 `provider: 'none'` 与 `scan: true`。
