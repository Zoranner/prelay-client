# MCP 扩展安装设计

## 目标

在智能体页面的扩展库中安装 MCP 扩展。安装过程只负责读取固定版本的 MCP 包清单、生成各宿主的官方 MCP 配置，并记录包版本状态；环境变量与密钥取值不由本应用保存。

本文档描述当前实现，尚未落实的事项集中列在文末。

## 边界

- 安装目标为 Codex CLI、ChatGPT 与 OpenCode；Codex CLI 与 ChatGPT 共用一套 MCP 配置，属于同一个安装目标。
- 支持 `stdio` 和 HTTP MCP。
- `stdio` 只写入命令配置，不启动命令。
- 不下载 `uvx`、`bunx`、`npx` 或其他运行时。
- 不下载包内二进制，不解压制品，不执行远程安装脚本。
- 不提供 MCP「全部更新」。
- 工作目录使用宿主默认：清单不声明工作目录，安装界面不展示、不提供输入。
- 安装状态只服务于包版本判断，不保存客户端列表、用户输入或密钥取值。

## 清单职责

MCP 包中的 `server.json` 是 MCP Registry 服务发现和发布元数据，不是 Codex、ChatGPT 或 OpenCode 的最终运行时配置。

Prelay 读取清单后，转换为内部安装描述。内部安装描述只在当前安装请求中存在，不写入本地安装状态以外的位置。

清单需要提供：

- MCP 服务身份；
- `stdio` 命令及参数；
- 所需环境变量名称；
- HTTP endpoint 和请求头声明。

版本和 commit 由扩展库的固定版本提供，不要求清单声明。

清单不得包含：

- 工作目录（固定使用宿主默认）；
- API Key 实际值；
- 密码；
- Cookie；
- OAuth Token；
- Authorization 实际值；
- 用户机器上的绝对路径；
- 某个宿主专用的最终配置字段。

## 宿主安装目标

### Codex CLI

```text
配置：~/.codex/config.toml
状态：~/.codex/.prelay/mcp.json
```

ChatGPT 与 Codex CLI 属于同一个安装目标：同时选择两者只写入一次配置、只写入一份状态记录；状态计算时同一份记录可以同时代表两个客户端。只检测到 ChatGPT 时，安装仍然写入 Codex 配置。

写入 `mcp_servers` 下以服务名命名的表：

```toml
[mcp_servers.filesystem]
command = "uvx"
args = ["mcp-server-filesystem", "%USERPROFILE%\\Documents"]
env_vars = ["API_KEY"]
enabled = true
# 清单声明超时时间时写入 tool_timeout_sec（毫秒换算为秒，向上取整）
```

HTTP 传输改为写入 `url` 和 `env_http_headers`（请求头名到环境变量名的映射），其余字段相同。

覆盖写入只替换该服务名对应的表，保留同一文件中的其他内容。

### OpenCode

```text
配置：~/.config/opencode/opencode.jsonc
状态：~/.config/opencode/.prelay/mcp.json
```

写入 `mcp` 下以服务名命名的对象（`stdio`）：

```json
{
  "mcp": {
    "filesystem": {
      "type": "local",
      "command": ["uvx", "mcp-server-filesystem"],
      "environment": { "API_KEY": "{env:API_KEY}" },
      "enabled": true
    }
  }
}
```

HTTP 传输改为写入 `type: "remote"`、`url` 和 `headers`（请求头名到 `{env:变量名}` 的映射）。超时时间按毫秒写入 `timeout`。

写入保留文件中的其他用户内容；解析后重新序列化可能丢失原有 JSONC 注释与排版。

## 宿主联动

安装界面按安装目标联动选择：

```text
勾选 Codex CLI
    -> 同时勾选已检测到的 ChatGPT

取消共享目标中的任一客户端
    -> 同时取消 Codex CLI 与 ChatGPT
```

底层按安装目标去重，只执行一次 Codex 配置写入和一次状态写入。OpenCode 不参与联动，独立选择。

## 本地状态

状态文件按扩展库包名记录版本和清单快照：

```json
{
  "filesystem-mcp": {
    "serverName": "filesystem",
    "version": "v1.0.0",
    "commitSha": "abc123",
    "manifest": {
      "name": "filesystem",
      "transport": {
        "type": "stdio",
        "command": ["uvx"],
        "cwd": null,
        "environment": { "API_KEY": "API_KEY" },
        "enabled": true,
        "timeoutMs": null
      }
    }
  }
}
```

- `serverName` 用于服务名改名后清理旧配置；
- `version` 与 `commitSha` 用于判断安装、更新和已安装；
- `manifest` 快照用于判断宿主配置是否仍与 Prelay 写入的一致，缺失时按「更新」处理。

状态文件不记录客户端列表、用户输入、环境变量取值和密钥。

宿主由状态文件路径确定，不增加 `client` 字段。

## 扩展列表状态计算

获取 MCP 扩展列表后，以扩展库返回的包名作为当前清单，与对应宿主状态文件中的包名比较。

状态规则：

```text
本地没有包记录
    安装

本地有包记录，但版本或 commit 不一致
    更新

本地有包记录，宿主配置与清单快照不一致
    更新

本地有包记录，部分宿主有记录
    补装

本地有包记录，版本、commit 和宿主配置都一致
    已安装

本地有包记录，但扩展库当前没有这个包
    删除状态文件中的包记录
```

扩展包从扩展库消失时，只删除本地状态记录：

- 不删除 Codex 配置；
- 不删除 OpenCode 配置；
- 不删除命令；
- 不删除用户环境变量；
- 不删除用户文件。

包改名时，旧包不在本地清单中，新包也没有本地记录，因此：

```text
旧包状态记录被清理
新包显示安装
不显示更新
不自动删除旧 MCP 配置
```

服务名改名的再次安装中，仅在旧配置仍与 Prelay 保存的清单快照一致时删除旧服务名配置。

## 环境变量输入

清单用同名映射声明需要的环境变量名称：

```json
{
  "environment": {
    "API_KEY": "API_KEY",
    "GITHUB_TOKEN": "GITHUB_TOKEN"
  }
}
```

安装界面按清单逐项展示变量名，并说明取值由用户本机环境提供；界面不收集、不校验、不回显变量取值。安装请求只携带扩展包、目标客户端和覆盖标记，宿主配置写入的是环境变量名称引用：

```text
Codex CLI   env_vars = ["API_KEY"]
OpenCode    "API_KEY": "{env:API_KEY}"
```

实际值由用户本机环境提供。密钥值不进入：

- `server.json`；
- `.prelay/mcp.json`；
- Prelay 日志；
- 安装结果消息；
- 测试夹具。

在界面输入并保存密钥值需要额外接入 Windows Credential Manager 或其他本地安全凭据存储，不属于当前安装模型；在该能力落地前，界面不提供取值输入框。

## 用户目录变量

清单的启动命令与参数可以使用 Prelay 定义的用户目录模板，安装时由客户端展开：

```text
%USERPROFILE%    -> 当前用户目录
%APPDATA%        -> 当前用户 Roaming 目录
%LOCALAPPDATA%   -> 当前用户 Local 目录
%PRELAYHOME%     -> Prelay 客户端程序目录（主程序所在目录）
```

例如清单中的 `%USERPROFILE%\Documents` 在写入宿主配置时展开为当前用户目录下的路径；`%PRELAYHOME%\tools\imagegen.exe` 展开为客户端程序目录下的工具路径。`%PRELAYHOME%` 是客户端自定义变量，值不来自系统环境，由客户端进程在安装时解析为主程序所在目录，因此工具文件随安装包落位后不需要再复制。

双百分号表示转义，不展开：

```text
%%USERPROFILE%%\Documents
```

安装后保留为：

```text
%USERPROFILE%\Documents
```

展开规则作用于启动命令与命令参数，不作用于环境变量名称；出现未识别的 `%变量%` 时安装在写入前失败，不把字面模板写入宿主配置。安装状态中的清单快照保留原始模板，宿主配置的漂移判断按展开后的结果比较。

## 参数展示

参数逐项展示，不把多个参数拼接成一行：

```text
启动命令：uvx

启动参数：
  mcp-server-filesystem
  %USERPROFILE%\Documents
  --mode
  readonly
```

参数是否包含用户目录变量不影响展示，安装界面按清单原样展示。

## 安装预览

MCP 安装抽屉展示：

- MCP 服务名称与版本；
- 传输方式；
- 启动命令；
- 每个参数；
- 所需环境变量；
- HTTP 地址和请求头映射；
- 安装目标。

不展示：

- 工作目录（固定使用宿主默认）；
- `enabled` 字段；
- 环境变量实际值；
- 密钥；
- 不相关宿主配置；
- 不必要的内部状态字段。

安装动作本身代表写入并启用目标配置。用户需要停用 MCP 时，在对应智能体的 MCP 配置中处理，不通过扩展包清单控制。

## 安装流程

```text
用户进入 MCP 扩展分类
    -> 请求扩展库 MCP 包列表
    -> 对比对应宿主的 mcp.json
    -> 显示安装 / 更新 / 补装 / 已安装
    -> 用户点击安装
    -> 打开安装抽屉，读取固定版本 server.json
    -> 展示服务身份、启动配置、所需环境变量名称和安装目标
    -> 用户确认
    -> 按宿主生成官方 MCP 配置
    -> 原子写入配置
    -> 写入包版本和 commit
    -> 刷新扩展列表和智能体条目
```

安装器不执行 MCP 命令，也不验证 MCP 服务是否能够启动。

## 配置冲突

如果目标宿主已经存在同名 MCP 配置：

- 第一次安装必须提示配置冲突；
- 用户明确确认后才允许覆盖；
- 覆盖只替换对应 MCP 配置项；
- 保留同一配置文件中的其他内容；
- 不覆盖其他宿主配置；
- 不删除用户环境变量。

跨多个宿主的安装先完成全部目标的冲突和格式预检，再执行写入。

## 错误处理

以下情况必须在写入前失败：

- `server.json` 无法解析；
- 包版本与返回的固定 commit 不一致；
- 清单声明了不支持的 transport；
- 命令为空；
- 清单声明了工作目录（工作目录固定使用宿主默认）；
- 环境变量名称非法，或不是同名引用；
- 命令参数中出现明文凭据选项；
- 启动命令或参数中出现未识别的目录变量；
- HTTP 地址不安全（非 HTTP/HTTPS、带 query/fragment 或内嵌凭据）；
- 宿主配置文件无法解析；
- 目标配置存在且用户未确认覆盖。

错误不得执行命令，不得下载运行时，不得把密钥写入日志。

写入失败后必须刷新扩展列表，使安装状态反映实际本地状态。

## 迁移

Skill 的既有迁移规则保持不变：

```text
~/.agents/.prelay/skills/<旧状态文件>
    -> ~/.agents/.prelay/skill.json
```

MCP 没有已发布的旧状态格式。本地状态缺少 `manifest` 快照的旧记录按「更新」处理；未来如果发现中间版本状态文件，只迁移包名、版本和 commit，不迁移完整 manifest、宿主列表或输入值。

联调用的本地模拟入口已按本节移除：`extensions_mcp_test_package` 命令、扩展库 MCP 分类的临时「安装」按钮、命令注册与本地模拟包夹具都已删除，安装只从真实扩展库列表发起。

## 当前未落实

- 服务端 MCP 清单校验收紧的提交尚未推送与部署。

## 验收标准

- Codex CLI 的 `stdio` 和 HTTP MCP 只写入 `mcp_servers` 下对应服务表；
- Codex CLI 与 ChatGPT 同时安装时，只写入一次共享配置和一份状态记录；
- 只检测到 ChatGPT 时，安装仍然写入 Codex 配置；
- OpenCode 可以独立选择，只写入 `mcp` 下对应服务对象；
- 命令参数中的 `%USERPROFILE%` 等模板在写入时展开，`%%` 转义保留为字面量；
- 启动命令与参数都参与模板展开，`%PRELAYHOME%` 指向客户端程序目录，未识别的变量在写入前失败；
- MCP 列表为空时，正式扩展列表显示空状态；
- 启动命令与参数逐项展示；
- 环境变量按清单逐项展示变量名，界面不收集取值；
- 实际密钥不进入包清单、状态文件或日志；
- 本地状态只记录包版本、commit 和无秘密清单快照；
- 扩展包从扩展库消失时只删除状态记录，不动宿主配置；
- 包改名时显示安装，不显示更新；
- 不执行 MCP 命令，不下载运行时；
- Codex CLI、ChatGPT、OpenCode 的配置文件都保留无关用户配置；
- 所有 MCP 安装行为都有对应的原生和前端测试。
