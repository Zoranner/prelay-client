//! Prelay 图像生成 MCP 服务端入口。
//!
//! 该可执行文件由智能体客户端（Codex 等）作为 stdio MCP 服务启动，不依赖 Tauri 运行时。

fn main() {
    std::process::exit(prelay_client::imagegen::run_stdio());
}
