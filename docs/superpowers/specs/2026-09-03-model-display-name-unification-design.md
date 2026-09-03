# 模型显示名称统一设计

## 背景与目标

当前系统在供应商、接入点、智能体配置、活动和仪表盘中同时使用模型 ID 作为内部值和界面文字。ProviderCatalog 已经为语言模型和图像生成模型维护稳定的 `id` 与 `display_name`，但管理 API 的业务响应没有把这层信息带到客户端，导致各页面重复推断或直接显示 ID。

本次改动建立一条服务端统一解析、客户端统一消费的模型显示链路：

- 模型 ID 是唯一的保存、查询、筛选和路由值。
- ProviderCatalog 是显示名称的唯一权威来源，不在数据库保存显示名称快照。
- 找不到目录项时，响应中的显示名称回退为模型 ID，保证历史数据和自定义 ID 仍可用。
- 接入点中同一对外模型组的多个供应商路由继续分别保存；显示目录按模型组只呈现一项。

## 契约模型

`CatalogLanguageModelResponse` 与 `CatalogImageGenerationModelResponse` 已经是完整的模型目录结构，包含 `id`、`display_name` 和后续配置可能需要的能力字段。本次不再新增精简的模型引用类型，客户端和保存配置流程直接复用这两个目录类型。

`CatalogProviderResponse.language_models` 与 `image_generation_models` 继续使用 `Vec<String>` 作为供应商支持模型的关系集合；客户端通过全局 `ProviderCatalogResponse` 按 ID 关联到完整目录对象。这样不会在每个供应商响应中重复嵌入完整模型，也不会制造第二套模型事实。

业务响应增加显示字段，但不改变现有 ID 字段的含义：

- `ProviderModelResponse` 增加 `display_name`，由服务端根据 `model_name` 解析。
- `EndpointModelResponse` 增加 `display_name`，由服务端根据对外 `model_name` 解析；`upstream_model` 仍只表示实际上游 ID。
- `ActivitySummary` 增加 `model_requested_display_name` 和 `model_upstream_display_name`。
- `ModelStatsSummary` 增加 `model_requested_display_name`。

活动和统计的显示字段与对应 ID 同为 `Option`；ID 为空时显示名称也为空。这样不会把“未知模型”伪装成目录模型，也不会改变现有排序、聚合或筛选键。

服务端新增一个返回完整 `ProviderCatalogResponse` 的目录入口（复用现有 `ProviderCatalog::response`）；供应商清单仍只传输模型 ID，完整模型对象由目录入口统一提供。

## 服务端职责与数据流

`prelay-server::provider_catalog::ProviderCatalog` 提供统一解析入口，输入模型 ID，返回已有的语言或图像目录对象；未找到时仅在展示字段中回退为原 ID。业务 storage 层不自行读取 TOML，也不从数据库复制目录名称。

响应组装位置负责调用该解析器：

- Provider storage 在生成每个 `ProviderModelResponse` 时解析 `model_name`。
- Endpoint storage 在生成每个 `EndpointModelResponse` 时解析对外 `model_name`。
- Activity storage 在读取活动时分别解析请求模型和上游模型；持久化表结构和写入流程不变。
- Stats storage 在生成模型统计行时解析分组键 `model_requested`；SQL 仍按 ID 聚合。

目录缺失只影响显示字段，不使管理查询失败。解析器应覆盖语言模型和图像生成模型 ID 相同的情况，优先按业务场景明确的目录类型，通用回退仍为原 ID。

## 客户端消费规则

`prelay-client` 在 relay store 中保存完整 `ProviderCatalogResponse`，按模型 ID 建立语言/图像目录索引；所有页面通过同一索引取得目录对象和显示名称，避免各页面各自拼装标签。供应商、接入点、活动和统计响应中的显示字段仍由服务端统一补充，客户端缺字段时再用目录索引或 ID 回退。

- 供应商表单：目录供应商的模型引用用于 `label = display_name`、`value = id`；提交时只发送选中的 ID。编辑已有供应商时，响应中的 `ProviderModelResponse.display_name` 只用于显示，模型数组仍取 `model_name`。
- 接入点：列表和表单显示 `EndpointModelResponse.display_name`，分组键和提交值继续使用 `model_name`/上游 ID。相同对外模型 ID 的不同供应商映射合并为一个模型组，组内保留所有路由。
- 智能体配置：Codex/OpenCode 的选择项标签使用模型显示名称，保存的 `model`、`modelName` 和路由字段使用模型 ID。接入点模型项同时携带已解析的目录模型对象供本地配置写入；Codex 生成的模型目录项中，`slug` 使用 ID，`display_name` 和能力字段优先来自目录对象，目录缺失时沿用现有模板并以 ID 作为显示名称。
- 活动页：模型列优先显示对应 display name，必要时用 ID 作为辅助信息；筛选参数仍提交模型 ID。
- 仪表盘：模型统计名称使用 display name，统计查询和排序仍按 ID/原统计值执行。

客户端对旧服务端响应保持兼容：缺少新增字段时按目录索引或现有 ID 作为显示名称；供应商模型项仍为字符串时直接按 ID 关联目录。该兼容逻辑只用于过渡读取，不产生新的保存格式。

## 错误、兼容与迁移边界

- 不增加数据库列、索引或迁移；现有活动、供应商模型和接入点路由数据无需回填。
- 不改变模型 ID 的大小写、空白规范、主键或路由唯一约束。
- 新字段采用与现有 Rust/TypeScript 命名一致的 snake_case wire 名称；协议、服务端和客户端按顺序更新，避免三仓库复制 DTO。
- 目录加载失败仍按现有启动错误处理；单个模型 ID 不在目录中属于正常回退，不记录敏感信息。
- API Key、设备凭据、Endpoint Token 等机密不进入目录响应或新增字段。

## 验证策略

### 协议

- 为已有完整目录模型、`CatalogProviderResponse` 和新增业务响应字段增加序列化单元测试。
- 验证旧字段仍保持原值，新增字段缺失/存在时的 JSON 行为符合兼容约定。

### 服务端

- 目录解析器覆盖语言模型、图像模型、同 ID 跨目录和未知 ID 回退。
- Provider、Endpoint、Activity、ModelStats 响应测试断言 display name 来自目录且 ID 不变。
- 现有路由唯一性、统计聚合和活动持久化测试继续通过，确认没有按显示名称聚合或路由。

### 客户端

- 归一化模型引用和模型组选项的定向测试，覆盖旧字符串目录响应、同名多供应商路由和未知模型回退。
- 供应商、接入点、智能体、活动、仪表盘页面测试分别断言 label 使用 display name、提交/筛选值使用 ID。
- 按仓库约定执行 Bun 类型检查、格式检查和相关测试；Rust 仓库执行 `cargo fmt --all` 与 `cargo clippy --all-targets --all-features -- -D warnings`。

## 实施顺序

1. 在 `prelay-protocol` 定义模型引用和响应字段并完成协议测试。
2. 更新 `prelay-server` 的目录解析器、四类 storage 响应和服务端测试。
3. 更新 `prelay-client` 的类型、集中归一化工具、供应商/接入点/智能体/活动/仪表盘消费及客户端测试。
4. 分别复查三仓库 diff 和 submodule 指针；本次不推送、不发布、不修改数据库部署状态。
