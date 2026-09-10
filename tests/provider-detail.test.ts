import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

test("编辑供应商通过原生命令回填创建人可见的密钥", () => {
  const page = read("app/pages/providers.vue");
  const command = read("src-tauri/src/commands/providers.rs");
  const application = read("src-tauri/src/app/mod.rs");
  const relayCommand = read("app/composables/useRelayCommand.ts");

  expect(page).toContain('invokeCommand<Provider>("providers_get"');
  expect(page).toContain("editable = { ...provider, ...detail }");
  expect(command).toContain("pub async fn providers_get(");
  expect(command).toContain('.get(&format!("/api/providers/{provider_id}"))');
  expect(command).toContain("ProviderResponse");
  expect(application).toContain("crate::commands::providers::providers_get");
  expect(relayCommand).toContain('"providers_get"');
});
