import { expect, test } from "bun:test";

import type { Provider } from "../app/stores/relay";
import { providerProtocolOptions } from "../app/utils/providerCapabilities";
import {
  PROVIDER_TEMPLATE_GROUPS,
  protocolLabel,
  providerTemplateForType,
} from "../app/utils/providerTemplates";

const provider = (overrides: Partial<Provider> = {}): Provider => ({
  id: "provider-1",
  name: "测试供应商",
  provider_type: "openai_compatible",
  base_url: "https://api.example.com/v1",
  api_key: "sk-provider-key",
  api_key_masked: "sk-***",
  upstream_protocols: ["responses", "anthropic"],
  capabilities: {
    upstream_protocols: ["responses", "anthropic"],
    protocol_base_urls: {
      responses: "https://responses.example.com/v1",
      openai: null,
      anthropic: "https://anthropic.example.com/v1",
    },
    tool_calls: true,
    reasoning: false,
    tool_choice: true,
    parallel_tool_calls: false,
    system_messages: true,
    structured_outputs: true,
    streaming_usage: false,
    max_context_tokens: 128000,
    max_output_tokens: 8192,
  },
  models: [],
  created_at: "2026-08-18T00:00:00Z",
  ...overrides,
});

test("协议测试使用服务端解析的上游协议能力", () => {
  expect(providerProtocolOptions(provider())).toEqual([
    "responses",
    "anthropic",
  ]);
});

test("协议测试不再自行从供应商类型推导默认协议", () => {
  expect(
    providerProtocolOptions(
      provider({ provider_type: "anthropic", upstream_protocols: ["openai"] }),
    ),
  ).toEqual(["openai"]);
});

test("供应商表格中的协议按 Chat Completions、Responses、Anthropic 排序", () => {
  expect(
    providerProtocolOptions(
      provider({ upstream_protocols: ["responses", "anthropic", "openai"] }),
    ),
  ).toEqual(["openai", "responses", "anthropic"]);
});

test("供应商表格保留图像生成协议", () => {
  expect(
    providerProtocolOptions(
      provider({
        upstream_protocols: ["images_generations", "openai"],
      }),
    ),
  ).toEqual(["openai", "images_generations"]);
});

test("图像协议使用与其他协议同层的 API 名称", () => {
  expect(protocolLabel("images_generations")).toBe("Images Generations");
});

test("DeepSeek 模板默认支持 Responses 协议", () => {
  expect(providerTemplateForType("deepseek")?.protocols).toEqual([
    "responses",
    "openai",
    "anthropic",
  ]);
});

test("DeepSeek 位于 API 服务首位", () => {
  const apiServices = PROVIDER_TEMPLATE_GROUPS.find(
    (group) => group.label === "API 服务",
  );

  expect(apiServices?.options[0]?.value).toBe("deepseek");
  expect(apiServices?.options[1]?.value).toBe("bailian");
});

test("API 服务使用规范的平台名称", () => {
  expect(providerTemplateForType("deepseek")?.label).toBe("DeepSeek 开放平台");
  expect(providerTemplateForType("kimi")?.label).toBe("Kimi API 开放平台");
  expect(providerTemplateForType("minimax")?.label).toBe("MiniMax API 平台");
  expect(providerTemplateForType("zhipu")?.label).toBe("智谱 BigModel 平台");
});

test("API 服务默认使用 Chat Completions 协议地址", () => {
  expect(providerTemplateForType("deepseek")?.baseUrl).toBe(
    "https://api.deepseek.com/v1",
  );
  expect(providerTemplateForType("qwen")?.baseUrl).toBe(
    "https://dashscope.aliyuncs.com/compatible-mode/v1",
  );
  expect(providerTemplateForType("kimi")?.baseUrl).toBe(
    "https://api.moonshot.ai/v1",
  );
  expect(providerTemplateForType("zhipu")?.baseUrl).toBe(
    "https://open.bigmodel.cn/api/paas/v4/",
  );
  expect(providerTemplateForType("minimax")?.baseUrl).toBe(
    "https://api.minimaxi.com/v1",
  );
});

test("GoToken 位于套餐服务首位并使用官网基础地址", () => {
  expect(PROVIDER_TEMPLATE_GROUPS[0]?.options[0]).toMatchObject({
    value: "gotoken",
    label: "GoToken 套餐",
    providerType: "gotoken",
    baseUrl: "https://gotoken.cc",
    protocols: ["responses", "openai", "anthropic", "images_generations"],
    protocolBaseUrls: {
      openai: "https://gotoken.cc/v1",
      anthropic: "https://gotoken.cc/v1",
      images_generations: "https://gotoken.cc/v1",
    },
  });
});

test("供应商预设不重复填写与 Base URL 相同的协议地址", () => {
  for (const group of PROVIDER_TEMPLATE_GROUPS) {
    for (const template of group.options) {
      for (const protocolBaseUrl of Object.values(template.protocolBaseUrls)) {
        expect(protocolBaseUrl).not.toBe(template.baseUrl);
      }
    }
  }
});
