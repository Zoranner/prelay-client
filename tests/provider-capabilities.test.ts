import { expect, test } from "bun:test";

import type { Provider } from "../app/stores/relay";
import { providerProtocolOptions } from "../app/utils/providerCapabilities";
import {
  protocolLabel,
  providerTemplateForType,
} from "../app/utils/providerTemplates";

const catalogProvider = {
  id: "example",
  name: "示例供应商",
  auth_scheme: "bearer",
  base_url: "https://api.example.com",
  protocols: [
    "chat_completions",
    "responses",
    "anthropic_messages",
    "images_generations",
  ],
  protocol_base_urls: [
    { protocol: "chat_completions", base_url: "https://api.example.com/v1" },
    {
      protocol: "anthropic_messages",
      base_url: "https://api.example.com/v1/messages",
    },
  ],
  language_models: ["chat-model"],
  image_generation_models: ["image-model"],
};

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

test("服务端目录供应商映射为表单协议、地址和模型", () => {
  expect(providerTemplateForType("example", [catalogProvider])).toEqual({
    value: "catalog:example",
    label: "示例供应商",
    providerType: "example",
    baseUrl: "https://api.example.com",
    protocols: ["openai", "responses", "anthropic", "images_generations"],
    protocolBaseUrls: {
      openai: "https://api.example.com/v1",
      anthropic: "https://api.example.com/v1/messages",
    },
    languageModels: ["chat-model"],
    imageGenerationModels: ["image-model"],
    models: ["chat-model", "image-model"],
  });
});

test("不提供自定义供应商模板", () => {
  expect(providerTemplateForType("openai_compatible", [])).toBeUndefined();
});
