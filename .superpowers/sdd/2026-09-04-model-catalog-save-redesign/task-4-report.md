# Task 4 报告：按服务端目录生成 Codex 模型配置

## 结果

- `CodexConnection::Prelay.models` 现在直接使用 `Vec<CatalogLanguageModelResponse>`，移除 `CodexEndpointModel` 和可缺失的 `catalog_model` 副本。
- Prelay 保存时逐项序列化完整目录对象，额外设置 `slug` 为目录 `id`；不再读取内置 `deepseek_models.json` 或进行模板回退。
- 默认模型校验按目录模型 `id` 执行。
- Custom 连接不创建或更新 `models.json`，也不清除已有 `model_catalog_json`。
- 目录文件成功原子写入后才写入 `config.toml`；目录写入失败时配置保持原内容。
- 删除客户端内置 `deepseek_models.json`。

## 验证

- `cargo fmt --all --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml agents::settings::tests -- --nocapture`：9 passed
- `bun test tests/agents-flow.test.ts`：3 passed
- `git diff --check`

设置测试覆盖完整目录字段保留、必填字段缺失反序列化失败、Custom 不产生目录文件且保留既有路径，以及目录写入失败不落配置半成品。
