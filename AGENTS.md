# prelay-client

`prelay-client` 是 Windows 桌面管理台，采用 Tauri 2、Nuxt 4、Tailwind 4 和 `stellar-ui`。它只管理用户选定服务上的当前设备身份配置，不直接转发 AI 请求或保存 Provider API Key。

## 前后端边界

- Nuxt 页面经 `app/composables` 和 Tauri command 调用原生层；管理 API 请求、设备身份读取、device credential、连接设置及其生命周期只由 `src-tauri` 负责。不要让浏览器层直接请求管理 API 或持有凭据。
- 首次连接只保存服务地址。身份由 `machine_id + account_sid` 定位，设备凭据由服务端签发；`username` 仅用于显示。
- Provider、Endpoint、模型映射、Endpoint Token、统计和诊断的 DTO 以 `crates/protocol` 子模块为准。变更任何传输结构前，先检查 `prelay-protocol` 与 `prelay-server` 的兼容性。
- 未保存表单的模型发现和协议测试可使用临时已鉴权调用，但 API Key 不得持久化或回显；已保存 Provider 的 Ping 只检查地址可达性，不替代模型发现。

## UI 约定

- `app/` 是 Nuxt 源目录。页面保留工作台、活动、供应商、接入、设置和首次连接的既有职责，不新增平行页面或状态源。
- 复用 `stellar-ui` 组件、现有 CSS token 和 `@iconify-json/ph` 图标。不要手绘 SVG、把 API 错误吞掉，或为局部页面重建独立样式体系。
- 工作台是桌面软件而不是营销页面：保持紧凑表格、明确操作状态和固定的一屏工作区边界。涉及视觉改动时，以实际渲染验收，不以 typecheck 或结构测试替代。

## 开发与验证

- Node.js 相关操作只使用 Bun，不使用 `npm`、`npx` 或生成 `package-lock.json` 的命令。
- 首次准备依赖或协议子模块缺失时执行：

```text
bun install --frozen-lockfile
git submodule update --init --recursive
```

- 前端修改后在仓库根目录按需执行：

```text
bun test
bun run typecheck
bun run generate
```

- 修改 `src-tauri` Rust 代码后，在 `src-tauri/` 目录执行：

```text
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Tauri 开发入口为 `bun run tauri dev`；开发服务器固定使用 `18081`。除非任务需要运行验证，不启动桌面应用或改变用户现有进程。
