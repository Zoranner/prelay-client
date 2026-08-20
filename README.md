# Prelay Client

Prelay 的 Windows 桌面客户端，使用 Tauri 2、Nuxt 4 和 Tailwind 4 管理 Provider、Interface 与本机身份凭据。

## 开发

```text
bun install --frozen-lockfile
bun run tauri dev
```

原生层通过 `crates/protocol` Git 子模块依赖 `prelay-protocol`。首次克隆后初始化子模块，再执行 Rust 构建、测试或打包：

```text
git submodule update --init --recursive
```

## 验证

```text
bun test
bun run typecheck
bun run generate
```
