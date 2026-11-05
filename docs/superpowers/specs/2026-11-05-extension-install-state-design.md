# 扩展安装状态设计

## 目标

按安装域组织扩展状态。安装域是宿主，当前只有 `codex` 一个值；Codex CLI 与 ChatGPT 是同一宿主上的两个客户端入口，不产生独立状态。每个宿主下分插件状态和规则状态：插件是整体安装单元，规则是独立安装单元。

删除按制品类型分散的状态文件，取消独立的 MCP、Skill、Hook 安装记录。本文档描述当前实现。

## 边界

- 只覆盖 Codex 宿主。Codex CLI 与 ChatGPT 共用 `~/.codex`、`~/.agents` 下的宿主位置。
- 扩展目录仍由服务端 `/api/extensions/rules` 与 `/api/extensions/plugins` 提供，本文不改变目录协议。
- 状态只记录归属、版本、目标和完整性标识，不保存密钥、请求头、清单原文、宿主配置副本或制品内容。
- 外部进程（Codex 自身）对宿主文件的写入不受 Prelay 锁保护，只能做到写前读取、写后校验。
- 不支持旧客户端读取新状态。降级依赖迁移备份。

## 术语

- 扩展：安装库里的分类，分为插件和规则。
- 插件：整体安装单元。一个插件包含若干 Skill、MCP、Hook 制品，安装时一次写入，不单独安装某类制品。
- 规则：独立安装单元。安装结果为替换目标规则文档。
- 宿主：安装域枚举，决定目标集合和状态目录。当前只有 `codex`。
- 客户端入口：Codex CLI 与 ChatGPT。只用于安装检测和界面展示，不是状态维度。

规则与插件互斥。服务端分类时，存在 `server.json`、`skills/*/SKILL.md`、`hooks/hooks.json` 任一文件即为插件；这些都不存在、但存在 `AGENTS.md` 时才是规则。

## 状态分类

宿主条目和声明项都按下面四种状态描述，后续章节只引用这四种：

| 分类 | 定义 |
| --- | --- |
| 已管理 | 声明项的身份在宿主中精确匹配 |
| 缺失 | 声明项在宿主中找不到 |
| 被修改 | 身份由插件或规则声明，但宿主内容与声明不一致 |
| 未归属 | 宿主条目不在任何插件的声明里 |

不同制品的判定能力不同：

- MCP 以服务名为身份，可以区分“已管理”和“被修改”。
- 规则以目标路径为身份，可以区分“已管理”和“被修改”。
- Skill 只校验目标目录和名称，不校验文件内容；存在即“已管理”，不存在即“缺失”。
- Hook 只能按 handler 哈希精确匹配。哈希不匹配时，宿主条目记为“未归属”，插件声明项记为“缺失”；无法可靠区分“用户改过”和“另一个同名 Hook”，因此不标记“被修改”。

## 划分维度

外层是宿主，内层是安装单元，安装单元内部按目标记录。

- 宿主决定状态目录和可用目标；插件和规则是宿主下的安装单元。
- 客户端入口映射到宿主：`codexCli → codex`、`chatgpt → codex`。
- 当前两个入口的目标完全重合，按入口建目录会为同一批文件写两份状态，安装、卸载、迁移都要跨入口对账。
- 客户端入口只是宿主的视图，展示和操作可以按入口触发，但状态只属于宿主。
- 未来出现独立宿主时新增宿主目录，不和 `codex` 共用状态，也不需要在同一份文件里做宿主分支。

## 状态文件布局

Prelay 自己的状态全部放在 `~/.prelay`：

```text
~/.prelay/
├── hosts/
│   └── codex/
│       ├── plugins/
│       │   ├── superpowers.json
│       │   └── refinery-mcp.json
│       ├── rule.json
│       ├── state.lock
│       └── .migrated
├── packages/
│   └── <plugin>/<commit>/
└── backup/
    └── <timestamp>/
```

- `hosts/<host>/plugins/<plugin>.json`：一个插件一个文件。
- `hosts/<host>/rule.json`：该宿主的规则状态，内部按目标路径记录。
- `hosts/<host>/state.lock`：该宿主所有 Prelay 写入的互斥锁。
- `packages/<plugin>/<commit>/`：插件受管内容，目前只存放 Hook 脚本；跨宿主共享。
- `backup/<timestamp>/`：迁移前的旧状态备份，迁移完成并验证后可以清理。

不存在 `clients` 字段，不存在全局插件列表，也不存在按客户端划分的目录。

## 插件状态

一个插件一个文件：

```text
~/.prelay/hosts/codex/plugins/superpowers.json
```

```json
{
  "schemaVersion": 2,
  "name": "superpowers",
  "version": "v6.3.0-prelay.2",
  "commitSha": "0a98854bd9653a01e188ff567803a54457bc9b2a",
  "state": "installed",
  "artifacts": [
    {
      "kind": "skill",
      "target": "C:\\Users\\u\\.agents\\skills",
      "names": ["brainstorming", "writing-plans"]
    },
    {
      "kind": "mcp",
      "target": "C:\\Users\\u\\.codex\\config.toml",
      "serverName": "refinery",
      "manifestHash": "sha256:…"
    },
    {
      "kind": "hook",
      "target": "C:\\Users\\u\\.codex\\hooks.json",
      "event": "SessionStart",
      "matcher": "startup|resume|clear|compact|fork",
      "handlerHash": "sha256:…"
    }
  ]
}
```

规则：

- 宿主由目录决定，文件里不重复记录宿主；读取时校验路径与请求宿主一致。
- 文件名必须与 `name` 一致；不一致按损坏处理，不读取、不覆盖。
- `artifacts` 是插件的内容清单，对应安装包里的 `artifacts[]`；它不是独立安装项，不产生独立状态。
- 每个 artifact 只记录落到哪个 `target` 以及定位它需要的身份字段。
- `state` 只取 `installing`、`installed`、`removing`。
- 不保存 MCP 清单原文、请求头、命令环境变量值。
- 同一身份不能同时属于两个插件：Skill 用 `target + name`，MCP 用 `target + serverName`，Hook 用 `target + event + matcher + handlerHash`。

`manifestHash` 的定义：把 Prelay 写入 Codex 的 MCP 表转成规范化 JSON 后取 SHA-256。表内包含服务名、传输类型、命令、参数、环境变量名、URL、请求头名、启用状态和超时时间；对象键排序、UTF-8、无多余空白。不包含环境变量值和请求头值。读取时对宿主配置中的同一张表做同样计算，用于判断配置是否被修改。

`handlerHash` 的定义与现有安装逻辑一致：对 `event`、`matcher`、`type`、`command`、`commandWindows` 做规范化 JSON 后取 SHA-256。它只校验 handler 定义，不校验 `packages/.../hooks/` 下的脚本文件内容；脚本被修改不会让状态失效。

## 规则状态

每个宿主一个规则状态文件，内部按目标路径记录：

```text
~/.prelay/hosts/codex/rule.json
```

```json
{
  "schemaVersion": 1,
  "targets": {
    "C:\\Users\\u\\.codex\\AGENTS.md": {
      "package": "development-rules",
      "version": "v0.2.0",
      "commitSha": "444058208e16c715f3acccac8eae89672554cd99",
      "contentHash": "sha256:…",
      "state": "installed"
    }
  }
}
```

- 规则不是插件内容，不出现在任何插件文件里。
- 目标路径使用规范化绝对路径：Windows 统一分隔符、统一大小写，避免同一文件出现两条记录。
- 当前 Codex 宿主只有一个规则目标 `~/.codex/AGENTS.md`，所以只有一条记录；同一宿主内不拆分文件。
- 记录中的 `state` 只取 `installing`、`installed`、`removing`。
- 目标文件缺失或哈希不一致时，记录保留，状态按“缺失”或“被修改”处理；不自动删除记录。
- 用户在设置里主动修改规则并保存时，清除该目标的记录；外部修改只标记状态，不清除。

## 宿主与客户端入口

- 后端只保留一个映射：`codexCli → codex`、`chatgpt → codex`。进入状态、安装、卸载、扫描后不再出现入口分支。
- 从任一入口安装或卸载，改的是同一份插件状态和同一批宿主文件。
- 检测状态独立：只装 CLI、只装桌面端、两者都装，宿主的安装状态相同；界面按检测结果显示可用入口。
- 不存在“只给 Codex CLI 装、不给 ChatGPT 装”的状态，因为两者读取的是同一批文件。
- 宿主级写入完成后，同时失效该宿主所有入口的缓存；不能只刷新当前入口。
- 卸载确认文案必须说明它影响整个宿主，而不是只影响当前入口。
- 目录中的 `installedClients` 由宿主状态推导：宿主已安装时，映射到该宿主的已检测入口都会返回；没有检测到的入口不显示。

## 状态读取语义

- `installed`：正常状态，参与“已管理”判断。
- `installing`、`removing`：视为未完成，目录和列表显示 `partial`；保留状态文件和安装计划，允许用户重新安装或执行清理，不自动续装。
- `installing`、`removing` 记录不参与已安装制品归属判断；只有 `installed` 记录参与 MCP、Skill、Hook 的归属和重复声明检查。
- 状态文件缺失：按没有安装处理。
- 状态文件损坏、`schemaVersion` 不支持、文件名与 `name` 不一致：该插件的产物按未归属处理，同时给出错误项；不静默当作未安装，也不使用其他插件的记录兜底。
- 未知的更高 `schemaVersion`：整个宿主状态进入只读模式，拒绝写入，提示升级客户端。
- 同一宿主内两个已安装插件声明同一个 artifact 身份：读取阶段报告冲突，两个插件都不参与归属判断，直到冲突解决。
- 已安装插件与未完成记录声明同一个 artifact 身份：未完成记录不参与归属判断，不产生冲突；未完成记录仍显示为 `partial`。
- 归属冲突未解决时拒绝写入整个宿主，避免在归属不明确时覆盖内容；这不改变版本升级触发的只读模式。
- 不同宿主声明同一个目标路径：读取阶段报告冲突，相关宿主进入只读模式，拒绝写入。

## 归属判断

| 产物 | 已管理 | 缺失 | 被修改 |
| --- | --- | --- | --- |
| 插件 Hook | `target + event + matcher + handlerHash` 精确命中 | 声明项在目标中不存在 | 不判定；哈希不匹配的宿主条目记为未归属 |
| 插件 MCP | `target + serverName` 命中，且宿主配置与清单字段一致 | 服务名不存在 | 服务名命中但配置不一致 |
| 插件 Skill | `target + name` 命中，目录存在 | 目录不存在 | 不校验内容 |
| 规则 | 目标路径命中且文档哈希与 `contentHash` 一致 | 目标文件不存在 | 目标存在但哈希不一致 |

未归属的宿主条目按个人配置展示，不自动认领。扫描只读状态，不写状态，也不根据文件路径反推归属。

## 智能体界面投影

插件和本地制品不是同一类数据，界面按类型分别呈现：

| 类型 | 呈现 | 身份 | 操作 |
| --- | --- | --- | --- |
| 插件 | 安装包条目列表：名称与版本、来源、包含内容、位置、操作 | 插件名 | 卸载整个插件 |
| MCP | 条目列表：名称、来源、类型事实、目标、位置、操作 | `target + serverName` | 删除宿主配置项 |
| Skill | 条目列表：名称、来源、文件数、说明、位置、操作 | `target + name` | 删除技能目录 |
| Hook | 按事件分组的处理程序列表：匹配条件、来源、执行命令、位置、操作 | `event + matcher + handlerHash` | 删除对应 handler |
| 规则 | 独立编辑器，不在条目列表中展示 | 目标规则文件 | 编辑并保存 |

所有条目使用同一副骨架：左上正方形类型图标、身份、来源、类型事实、详情、右上操作。插件子内容常驻展示，不提供展开折叠；插件内的 Skill、MCP、Hook 每组固定一行，不换行，超出宽度时由 `TagGroup` 汇总为 `+N`。顶层 Skill、MCP、Hook 条目的详情均为单行省略，完整值通过 `title` 提供。路径位于名称后的文件夹图标中，操作区仅在条目悬停或聚焦时显示。

插件已经声明的 Skill、MCP、Hook 不在对应本地条目列表里重复展示；插件卸载时统一清理。插件的 Skill 内容中，以 `.` 开头的内部 Skill 不展示。不可解析且没有可执行清理动作的宿主条目不进入列表投影。Hook 没有名称：事件用于分组，matcher 是可见身份，执行命令是详情内容。`handlerHash` 只作为后端精确归属和卸载匹配的内部稳定键，不作为界面字段展示。

列表结构的可交互原型见 `docs/prototypes/agent-item-list-prototype.html`。

## 配置读写边界

宿主维度对所有配置读写都成立，但写入策略取决于数据所有者，不能共用一套。

| 数据 | 所有者 | 读取 | 写入 |
| --- | --- | --- | --- |
| `~/.prelay/hosts/<host>/...` | Prelay | 直接读，损坏报错 | 整文件原子替换，带 `schemaVersion`，持锁 |
| `~/.codex/config.toml` | 宿主 | 解析 TOML，失败报错 | 只改 Prelay 拥有的键，保留未知字段和排版 |
| `~/.codex/auth.json` | 宿主凭据 | 只读需要的键 | 只改自己的键，原子替换，不进入状态或日志 |
| `~/.codex/hooks.json` | 宿主 | 解析后按 handler 识别 | 增删自己的 handler，保留其他 handler |
| `~/.codex/AGENTS.md` | 宿主规则 | 读取并计算哈希 | 规则安装时整文档替换 |
| `~/.agents/skills/*` | 宿主 | 检查目录存在性 | 按目录安装或删除 |

统一规则：

- 同一宿主下的两个客户端入口读写同一份配置，不按入口复制。
- 只写 Prelay 拥有的键或条目，未知内容保留；TOML 使用 `toml_edit` 修改而不是重新序列化整份文档。
- 写前读取，写后校验；内容未变化不写。
- 凭据不进入状态文件、日志或错误消息。
- 安装入口、管理状态这类派生视图在读取时计算，不落盘。

当前剩余缺口：设置读取把解析失败当作空配置，损坏的 `config.toml`、`auth.json` 会被当成缺省值；应改为报错。设置保存与扩展安装已经共用宿主级锁。

## 安装

### 预检与覆盖

安装前先预检每个目标，按目标现状决定行为：

| 目标现状 | 默认行为 | `overwrite=true` |
| --- | --- | --- |
| 不存在 | 安装 | 安装 |
| 由同一个插件管理，版本和 commit 都相同 | 重写（幂等） | 重写 |
| 由同一个插件管理，版本或 commit 不同 | 更新 | 重写 |
| 由另一个插件管理 | 冲突，终止 | 冲突，终止 |
| 未受管的个人内容 | 冲突，终止 | 覆盖 |
| 声明项缺失（宿主被手工删除） | 修复 | 重写 |

冲突在预检阶段终止整次安装，不写入任何宿主文件，也不创建插件状态。

覆盖成功后不保留持久备份。被覆盖的个人 Skill 目录、MCP 配置和规则文档无法在进程崩溃后恢复；同一进程内的可处理错误只能用内存前像恢复 MCP 条目。

### 安装流程

1. 预检全部制品，按上表确认每个目标。
2. 写入插件状态，`state` 为 `installing`，`artifacts` 即本次安装计划。
3. 按计划依次应用 Skill、MCP、Hook。
4. 删除旧记录中已不在新计划里的 artifact；被用户修改的旧 MCP 在写入前按冲突终止。
5. 全部成功后把 `state` 改为 `installed`。

## 卸载

### 预检与冲突

卸载前逐个检查 artifact：

| 现状 | 行为 |
| --- | --- |
| 已管理 | 删除 |
| 缺失 | 跳过并记录 |
| 被修改 | 冲突 |

任意一个 artifact 冲突时，整次卸载不删除任何内容，也不修改插件状态。用户需要恢复原内容或重新安装后再卸载。

### 卸载流程

1. 预检全部 artifact，发现冲突立即终止。
2. 把插件状态改为 `removing`。
3. 按 artifact 逐个删除。
4. 全部成功后删除插件状态文件。

删除过程中失败时保留 `removing`，标记 `partial`；下次启动只核对并报告，不自动续删。

## 规则安装与卸载

规则安装：

1. 预检目标：同一规则包可以更新；另一个包管理的目标或未受管的个人文档需要 `overwrite=true`。
2. 在 `rule.json` 写入目标记录，`state` 为 `installing`，同时记录包信息和计划写入的内容哈希。
3. 写入目标文档。
4. 把记录改为 `installed`。

规则状态清除：

1. 用户在规则编辑器保存了不同内容时，清除该目标的安装记录。
2. 不删除规则文档。

当前没有独立的规则包卸载命令或界面入口；规则包安装后由智能体规则编辑器修改，保存规则内容会清除该目标的安装记录。规则文件本身保留。

## 事务、失败与恢复

- 可处理错误：按已应用的 artifact 逆序回滚，删除本次新增的 Skill 目录、Hook 条目、MCP 条目；MCP 条目如果在同一进程内记录了前像，可以恢复。
- 不可回滚：`overwrite=true` 覆盖的个人 Skill 目录、MCP 配置、规则文档。失败时保留 `installing`，标记 `partial`，不假装成功。
- 进程崩溃：启动时发现 `installing` 或 `removing`，按状态中的计划核对宿主实际内容，标记 `partial`；不自动续装，也不自动删除无法确认的内容。
- 崩溃恢复只报告，不修复。用户可以重新安装覆盖，或先恢复宿主内容再执行卸载。

## 共享资源生命周期

- `packages/<plugin>/<commit>/` 跨宿主共享。删除前必须扫描所有 `hosts/*/plugins/<plugin>.json`，确认没有宿主再引用该插件和 commit。
- 插件更新先写入新 commit 目录，成功后更新状态；旧 commit 目录在最后一个引用消失后清理。
- 卸载只删除当前宿主的状态和 artifact；共享目录在最后一个引用消失后才删除。

## 锁与并发

- 每个宿主一个 `state.lock`，覆盖该宿主的状态文件和宿主配置写入：`config.toml`、`hooks.json`、`AGENTS.md`、`skills` 目录。
- 设置保存和扩展安装共用同一把宿主锁，不能各用一套。
- 迁移和跨宿主清理需要持有相关宿主锁，按固定顺序获取，避免死锁。
- 外部进程写宿主文件不受锁保护；写入前读取、写入后校验，发现变化时报冲突而不是覆盖。

## 迁移

迁移在首次读取宿主状态时执行一次，持有相关宿主锁；成功后写入 `hosts/<host>/.migrated`，后续不再重复执行。

来源优先级：

1. `extensions.json` 的包记录：版本、commit、目标入口。
2. `hooks.json`、`mcp.json`、`skill.json` 补齐缺失的 artifact 和哈希。
3. `rule.json` 生成规则目标记录。

合并规则：

- `codexCli` 与 `chatgpt` 合并到宿主 `codex`；版本和 commit 以包级记录为准，`targets` 有任意一个入口即视为该宿主已安装。
- 分制品状态的版本或 commit 与包记录不一致时，标记 `partial`，不自动选择。
- 规则型扩展只生成 `rule.json` 记录，不生成插件文件。
- `rule.json` 没有 `contentHash` 时，用迁移时的目标文件内容计算基线；该基线只代表迁移时点，之后被修改会显示为“被修改”。
- `skill.json` 与 legacy `skills/*.json` 同时存在时，合并技能名并去重。

执行步骤：

1. 把旧状态文件备份到 `~/.prelay/backup/<timestamp>/`，不只备份 `extensions.json`。
2. 生成新状态并写入。
3. 读回新状态，核对包、artifact、规则目标数量与旧记录一致。
4. 校验通过后删除旧状态文件并写入迁移标记；失败时保留迁移备份。

迁移失败时，新状态可能已经部分写入，因此不会继续按旧格式读取。用户需要使用迁移备份恢复，或在清理残留新状态后重试。

迁移只做单向转换。迁移完成后，旧文件不再被读取，也不再被写入。

## 版本与兼容

- 每个新状态文件带 `schemaVersion`。
- 迁移前的旧文件没有 `schemaVersion`，只由迁移流程处理，不参与常规读取。
- 读到高于客户端支持的版本时进入只读模式，拒绝写入并提示升级；不能覆盖。
- 不支持旧客户端读取新状态，降级只能使用迁移备份。

## 验证

- 插件可含 Skill、MCP、Hook 的任意组合；分别验证只有 Codex CLI、只有 ChatGPT、两者都检测到三种情况。
- 从 CLI 入口安装、从 ChatGPT 入口卸载，验证操作的是同一份宿主状态，且另一个入口的缓存同步失效。
- 覆盖矩阵：目标不存在、同插件同版本、同插件不同版本、另一个插件、未受管个人内容、声明项缺失。
- 插件更新时移除不再包含的 Skill、MCP、Hook，并保留仍被新计划声明的部分。
- 卸载矩阵：已管理、缺失、被修改，验证冲突时不删除任何内容。
- 规则：哈希匹配、哈希不匹配、包不匹配、目标缺失、安装中崩溃。
- 用户改过 Hook、MCP、Skill 后的安装与卸载。
- `overwrite=true` 覆盖后的失败和进程中断。
- 跨宿主共享 `packages` 的引用计数；卸载一个宿主不影响另一个宿主。
- 不支持的 `schemaVersion`、文件名与 `name` 不匹配、路径大小写差异、同一身份被两个插件声明、不同宿主声明同一目标。
- 设置保存与扩展安装并发写同一宿主配置。
- 旧状态迁移：仅有 `extensions.json`、仅有各制品状态、两者并存、分制品状态与包记录版本不一致、`rule.json` 无哈希、legacy `skills/*.json`。
- 结构门禁：单文件不超过 450 行。

## 重构范围与影响

重构对象是智能体配置子系统，不只是扩展状态文件。

直接重写：

- 扩展状态：`extensions/record.rs`、`extensions/hooks/state.rs`、`extensions/mcp.rs` 的状态部分、`extensions/skill_state.rs`、`extensions/rules.rs`。
- 安装与卸载：`extensions/install.rs`、`extensions/uninstall.rs`、`extensions/catalog.rs`、`extensions/checks.rs`。
- 新增：宿主状态模块、迁移逻辑、宿主级锁。

需要改签名或适配：

- 扫描：`agents/items.rs`、`agents/integrations/codex_cli.rs`、`agents/integrations/chatgpt.rs`。
- 模型：`agents/model.rs`，把客户端入口和宿主分开；`AgentItem` 增加 `hookHash`，作为未归属 Hook 的稳定身份。
- 命令：`commands/agents.rs`、`commands/extensions.rs`，内部把入口映射到宿主；`agents_remove` 增加可选 `hookHash`，其余入参保持。
- 设置：`agents/settings/*` 内部按宿主读写，`agent_settings_get(client)` 仍按入口返回同一份配置。

前端契约：

- `AgentClient`（`codexCli` / `chatgpt`）、`installedClients` 和安装请求的客户端列表不变。
- `AgentItem` 保留现有字段并增加 `hookHash`；Hook 的 `name` 是事件，不是独立名称。
- `installedClients` 仍按入口返回，只是后端改为从宿主状态推导：宿主已安装时，映射到该宿主的入口都视为已安装，未检测到的入口不显示。
- 安装选择、头像、章节、卸载入口不变；共享宿主的入口本来就是联动的一组。
- 条目展示按类型拆成独立视图：插件使用安装包条目列表，MCP、Skill、Hook 使用各自的本地条目列表；不再用一套表格列定义承载四种语义。

行为变化：

- 目录里的 `installedClients` 来源改成宿主状态推导，返回值仍是入口列表。
- 更新全部按宿主执行，不再按客户端重复安装。
- 从 Codex CLI 或 ChatGPT 任一入口卸载，清理的是整个宿主的安装。
- 安装选择仍可按入口展示，但共享宿主的入口保持联动。

不在范围内：

- `prelay-server` 与 `prelay-protocol` 的 DTO。
- Provider、Endpoint、接入、统计、身份认证。
- Codex 宿主自身的读写行为。

## 剩余风险

- 外部进程写宿主文件不受 Prelay 锁保护，只能在写入前后发现变化，无法阻止并发覆盖。
- 崩溃恢复只报告不修复，用户可能需要重新安装或手工恢复被覆盖的内容。
- 宿主状态存在但两个入口都未检测到时，目录显示未安装，界面没有清理入口；需要保留手动或命令行清理路径。
- 旧客户端读不了新状态，降级只能使用迁移备份。
- Hook 脚本文件内容不在状态校验范围内，脚本损坏不会被识别。
- 规则包没有独立卸载入口；规则目标只能通过规则编辑器清除记录。

## 未决边界

- 是否需要在插件状态中记录安装时间或来源仓库地址，当前不记录。
- 新宿主接入时新增 `hosts/<host>/` 目录；同一宿主内不按入口拆分状态。
- 迁移备份的保留时长和自动清理策略，当前只要求保留到迁移验证通过。
