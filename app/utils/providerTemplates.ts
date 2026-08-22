import type { UpstreamProtocol } from "~/stores/relay";

export type ProviderTemplate = {
  value: string;
  label: string;
  providerType: string;
  baseUrl: string;
  protocols: UpstreamProtocol[];
  protocolBaseUrls: Partial<Record<UpstreamProtocol, string>>;
  custom?: boolean;
};

export const PROVIDER_TEMPLATE_GROUPS: {
  label: string;
  options: ProviderTemplate[];
}[] = [
  {
    label: "套餐服务",
    options: [
      template(
        "gotoken",
        "GoToken 套餐",
        "gotoken",
        "https://gotoken.cc/v1",
        ["responses", "openai", "anthropic"],
        {
          responses: "https://gotoken.cc",
          openai: "https://gotoken.cc/v1",
          anthropic: "https://gotoken.cc/v1",
        },
      ),
      template(
        "kimi_code",
        "Kimi Code",
        "kimi_coding_anthropic",
        "https://api.kimi.com/coding",
        ["anthropic", "openai"],
        {
          openai: "https://api.kimi.com/coding/v1",
          anthropic: "https://api.kimi.com/coding",
        },
      ),
      template(
        "bigmodel_coding_plan",
        "GLM Coding Plan",
        "zhipu_coding",
        "https://open.bigmodel.cn/api/anthropic",
        ["openai", "anthropic"],
        {
          openai: "https://open.bigmodel.cn/api/coding/paas/v4",
          anthropic: "https://open.bigmodel.cn/api/anthropic",
        },
      ),
      template(
        "minimax_token_plan",
        "MiniMax Token Plan",
        "minimax_token",
        "https://api.minimax.io/anthropic",
        ["openai", "anthropic"],
        {
          openai: "https://api.minimax.io/v1",
          anthropic: "https://api.minimax.io/anthropic",
        },
      ),
    ],
  },
  {
    label: "API 服务",
    options: [
      template(
        "deepseek",
        "DeepSeek 开放平台",
        "deepseek",
        "https://api.deepseek.com/v1",
        ["responses", "openai", "anthropic"],
        {
          openai: "https://api.deepseek.com/v1",
          anthropic: "https://api.deepseek.com/anthropic",
        },
      ),
      template(
        "bailian",
        "阿里云百炼",
        "qwen",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        ["responses", "openai", "anthropic"],
        {
          responses: "https://dashscope.aliyuncs.com/compatible-mode/v1",
          openai: "https://dashscope.aliyuncs.com/compatible-mode/v1",
          anthropic: "https://dashscope.aliyuncs.com/apps/anthropic",
        },
      ),
      template("kimi", "Kimi API 开放平台", "kimi", "https://api.moonshot.ai/v1", [
        "openai",
      ]),
      template(
        "bigmodel",
        "智谱 BigModel 平台",
        "zhipu",
        "https://open.bigmodel.cn/api/paas/v4/",
        ["openai", "anthropic"],
        { anthropic: "https://open.bigmodel.cn/api/anthropic" },
      ),
      template(
        "minimax",
        "MiniMax API 平台",
        "minimax",
        "https://api.minimaxi.com/v1",
        ["responses", "openai", "anthropic"],
        {
          responses: "https://api.minimaxi.com/v1",
          openai: "https://api.minimaxi.com/v1",
          anthropic: "https://api.minimaxi.com/anthropic",
        },
      ),
    ],
  },
  {
    label: "其他服务",
    options: [
      template(
        "custom",
        "自定义",
        "openai_compatible",
        "",
        ["openai"],
        {},
        true,
      ),
    ],
  },
];

export function providerTemplateForType(providerType: string) {
  return PROVIDER_TEMPLATE_GROUPS.flatMap((group) => group.options).find(
    (item) => item.providerType === providerType,
  );
}

export function providerLabel(providerType: string) {
  return providerTemplateForType(providerType)?.label ?? providerType;
}

export function protocolLabel(protocol: UpstreamProtocol) {
  return protocol === "responses"
    ? "Responses"
    : protocol === "anthropic"
      ? "Anthropic Messages"
      : "Chat Completions";
}

function template(
  value: string,
  label: string,
  providerType: string,
  baseUrl: string,
  protocols: UpstreamProtocol[],
  protocolBaseUrls: Partial<Record<UpstreamProtocol, string>> = {},
  custom = false,
): ProviderTemplate {
  return {
    value,
    label,
    providerType,
    baseUrl,
    protocols,
    protocolBaseUrls,
    custom,
  };
}
