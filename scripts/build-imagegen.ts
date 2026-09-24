// 编译 imagegen 可执行文件，并复制为 Tauri 打包源文件。
import { $ } from "bun";
import { copyFile, mkdir } from "node:fs/promises";

const executableSuffix = process.platform === "win32" ? ".exe" : "";
const built = `src-tauri/target/release/imagegen${executableSuffix}`;
const bundleSource = `src-tauri/tools/imagegen${executableSuffix}`;

await $`cargo build --release --features imagegen --bin imagegen --manifest-path src-tauri/Cargo.toml`;
await mkdir("src-tauri/tools", { recursive: true });
await copyFile(built, bundleSource);
console.log(`imagegen 打包源文件已就绪：${bundleSource}`);
