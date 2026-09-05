# Codex 模型推理默认值统一设计

## 目标

让 Codex 与 ChatGPT 的推理强度完全遵循服务端模型目录：服务端维护模型支持的档位和默认档位，客户端只展示当前模型支持的选择，只有用户明确覆盖时才写入全局 `model_reasoning_effort`。

## 当前问题

- 客户端草稿固定初始化为 `high`，会覆盖服务端目录中的模型默认值。
- 设置表单固定展示 `low`、`medium`、`high`、`xhigh`，无法选择 `none`、`minimal`、`max`，也无法反映模型实际支持的档位。
- 客户端保存时无法区分“使用模型默认值”和“用户明确选择了某个值”。
- Tauri 原生保存入口没有校验推理覆盖值是否属于当前 Prelay 接入点模型的档位列表。

## 责任边界

### 服务端目录

`prelay-server/config/catalog/models/language.toml` 是模型推理事实源，提供：

- `reasoning_efforts`：模型支持的有序档位列表。
- `default_reasoning_effort`：必须属于非空的 `reasoning_efforts`，表示模型默认档位。

服务端已经负责目录加载和一致性校验，客户端不复制或推导这些事实。

### 客户端目录状态

客户端全局模型目录索引继续由 `app/utils/modelCatalog.ts` 管理。新增的推理适配逻辑只从 `CatalogLanguageModelResponse` 读取档位和默认值，不创建第二份模型能力目录。

### 用户设置状态

Codex 和 ChatGPT 草稿中的 `reasoningEffort` 使用空字符串表示“使用当前模型的目录默认值”，非空字符串表示用户显式覆盖。该状态只存在于编辑草稿和 Codex 配置文件，不新增本地持久化字段。

## 交互与状态流

1. 目录加载完成后，接入点模型选项携带完整 `CatalogLanguageModelResponse`。
2. 选择 Prelay 模型时，表单从该模型生成推理档位选项；第一项为“模型默认（目录默认档位）”，其值为空字符串。
3. 当前模型不支持推理档位时，推理选择器禁用并保持空覆盖值。
4. 切换模型时：
   - 当前显式覆盖仍在新模型支持列表中，则保留覆盖；
   - 否则清除覆盖，回到新模型目录默认值。
5. 自定义连接没有服务端模型目录，保留现有设置能力；其推理选择使用客户端允许的通用档位，不参与 Prelay 目录校验。
6. 打开已有配置时，读取 `model_reasoning_effort` 作为显式覆盖；字段缺失时显示当前模型的目录默认值，但草稿仍保持空覆盖。

## 保存与校验

### 前端

- Prelay 保存前确认目录已加载、默认模型属于接入点，且所有接入点模型都属于语言模型目录。
- 非空 `reasoningEffort` 必须属于当前模型的 `reasoning_efforts`。
- 保存请求在覆盖为空时省略 `reasoningEffort` 字段，避免把目录默认值写入用户配置。

### Tauri 原生层

- `CodexConnection::Prelay` 保存前重复校验非空推理覆盖值。
- 覆盖值不属于当前接入点模型时，返回稳定的配置错误，不写入 `config.toml`、`models.json` 或认证文件。
- 自定义连接不执行目录校验。

### 配置文件结果

- 使用目录默认值：删除 `model_reasoning_effort`，由 Codex 从 `models.json` 的 `default_reasoning_level` 取默认值。
- 用户显式覆盖：写入合法的 `model_reasoning_effort`。
- `models.json` 继续只由服务端目录模型生成，保留服务端默认思考档位，不写入 `null` 或客户端臆测的行为字段。

## 模块调整

- `app/utils/modelReasoning.ts`：集中提供档位标签、当前模型档位选项和覆盖值归一化。
- `app/utils/agentSettings.ts`：将草稿初始覆盖改为空，并在请求构造时省略空覆盖。
- `app/composables/useAgentSettings.ts`：向表单传递当前目录模型，执行前端覆盖校验和模型切换归一化。
- `app/components/agents/CodexSettingsForm.vue`：消费动态档位选项，展示目录默认值并处理模型切换。
- `src-tauri/src/agents/settings/codex.rs`：增加 Prelay 推理覆盖校验，保持写文件前失败不落盘。
- `src-tauri/src/agents/settings/tests.rs` 或独立测试模块：覆盖合法覆盖、非法覆盖、无覆盖和模型切换相关原生行为。
- `tests/model-catalog.test.ts` 或独立前端测试：覆盖档位选项、默认显示、空覆盖请求和非法覆盖阻断。

## 不在范围内

- 不修改服务端模型目录字段和协议 DTO。
- 不为每个用户或每个模型新增持久化配置结构。
- 不改变自定义供应商的连接方式、认证存储或其他 Codex 设置。
- 不将图像生成模型加入推理档位选择。

## 验收标准

- GPT-5.6 在设置面板显示完整档位，未显式覆盖时保存后不产生 `model_reasoning_effort`。
- DeepSeek/Kimi 三档模型显示 `low / high / max`，目录默认 `max` 能被正确展示和生成。
- 无推理档位模型不显示可选覆盖，保存结果不包含推理覆盖字段。
- 非法覆盖无法通过前端或直接调用 Tauri command 写入配置。
- 现有模型目录加载、接入点模型约束、Codex 配置生成和自定义连接行为保持不变。
