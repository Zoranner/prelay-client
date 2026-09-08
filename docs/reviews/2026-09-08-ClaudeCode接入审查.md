# prelay-client Claude Code 接入审查报告

## 评审范围

本次只审查提交 `395687c`“接入 Claude Code 配置管理”的工程实现、配置契约、状态链路和测试覆盖，不修改代码，不评价服务端 Anthropic 协议实现。

## 评审依据

- `prelay-client/AGENTS.md`：Nuxt 页面只经 Tauri 原生命令管理本地智能体配置；Endpoint Token 不得进入客户端持久化状态或日志。
- `prelay-client` 当前 Nuxt、Tauri、Bun、Rust 工具配置及现有智能体实现。
- `docs/superpowers/specs/2026-08-27-agent-extensions-design.md`：Claude Code 使用独立的 `.claude` 配置和 `CLAUDE.md` 规则目标。
- 提交 `395687c` 的 staged diff、当前分支状态和本轮静态检查结果。

## 总体结论

提交的目录分层和客户端适配边界基本清晰，Claude Code 已进入前端类型、Tauri 客户端发现、设置读写、规则和 Skill 目标链路，现有自动化门禁也全部通过。

本轮已修复设置回填和自定义接入点两个问题：

- Claude Code 使用服务根地址匹配已保存的 Prelay 接入点。
- Claude Code 不再显示不可用的“自定义”接入点选项。

## 检查情况

- `bun test`：151 项通过。
- `bun run typecheck`：通过。
- `bun run lint`：通过。
- `bunx prettier --check <本次涉及前端文件>`：通过。
- `cargo fmt --all -- --check`：通过。
- `cargo clippy --all-targets --all-features -- -D warnings`：通过。
- `cargo test --all-targets --all-features`：全部测试通过。
- 未执行真实 `claude` 命令、真实 Endpoint Token 请求和真实上游 Anthropic 联调；当前结论不包含运行环境验收。

## 主要问题

### Claude Code 保存后重新打开设置会丢失已选接入点

- **级别**：高
- **当前状态**：已修复。
- **问题定性**：违反功能链路的一致性要求，测试覆盖不足导致的风险。
- Claude Code 保存时将 `ANTHROPIC_BASE_URL` 写成去掉尾部斜杠的服务根地址，例如 `https://relay.example.test`，见 `src-tauri/src/agents/settings/claude_code.rs:29-32`。
- 前端统一使用 `managementBaseUrl()` 将服务地址规范化为 `https://relay.example.test/v1`，见 `app/composables/useAgentSettings.ts:43-47`。
- Claude Code 回填时直接用这个 `/v1` 地址与配置中的根地址比较，见 `app/composables/useAgentSettings.ts:362-366`。比较失败后会回退到 `__custom__`，因此保存成功后再次打开设置，接入点不会显示为原来的 Endpoint。
- 影响是用户看到的设置状态与实际 `settings.json` 不一致；后续再次保存还可能进入错误的自定义分支。
- 建议将 Claude Code 的地址规范化逻辑与 Codex/OpenCode 分开，或统一明确“存储根地址”和“管理 API `/v1` 地址”的语义，并为“保存后重新 hydrate”增加回归测试。

### “自定义”接入点对 Claude Code 不可用

- **级别**：高
- **当前状态**：已修复。
- **问题定性**：界面选项、前端连接模型和 Tauri 写入能力不一致。
- 修复前所有智能体的 `endpointOptions` 都包含 `__custom__`/“自定义”。
- Claude Code 表单只提供接入点和默认模型，没有自定义 Base URL 或 Token 输入，见 `app/components/agents/ClaudeCodeSettingsForm.vue:21-28`。
- Claude Code 的连接函数只处理已选 Prelay Endpoint，选中自定义时返回 `null`，见 `app/composables/useAgentSettings.ts:248-262`。
- Tauri 的 Claude Code 连接类型也只有 `Prelay`，见 `src-tauri/src/agents/settings/mod.rs:64-72`。
- 用户可以在 UI 中选择“自定义”，但无法提供必要参数；保存时只会写模型和规则，不能建立新的自定义 Claude Code 连接，也不会明确报错。
- 当前采用按客户端能力过滤，Claude Code 只显示已配置的 Prelay 接入点。

## 整改优先级建议

### 先修正设置契约

- 明确 Claude Code Base URL 的存储格式，修正 hydrate 比较逻辑。
- 明确是否支持 Custom connection，并让 UI 选项、DTO、Tauri 写入和测试保持同一契约。
- 增加保存、重新读取、再次保存的完整回归测试，至少覆盖 Prelay 和 Custom 两种路径。

### 修正后的扩展结论

- 复核后确认 Claude Code 的独立选择符合既定扩展设计：Codex CLI 与 ChatGPT 共享联动，Claude Code 独立。
- Skill 已纳入全部智能体；规则保持 Codex Host 联动，不作为本次缺陷处理。
