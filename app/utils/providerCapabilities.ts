import type { Provider, UpstreamProtocol } from "~/stores/relay";

const protocolValues: UpstreamProtocol[] = [
  "openai",
  "responses",
  "anthropic",
  "images_generations",
];

export function providerProtocolOptions(
  provider: Provider,
): UpstreamProtocol[] {
  return protocolValues.filter((protocol) =>
    provider.upstream_protocols.includes(protocol),
  );
}
