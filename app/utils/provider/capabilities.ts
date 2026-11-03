import type { Provider, UpstreamProtocol } from "~/types/relay";

const protocolValues: UpstreamProtocol[] = [
  "openai",
  "responses",
  "images_generations",
];

export function providerProtocolOptions(
  provider: Provider,
): UpstreamProtocol[] {
  return protocolValues.filter((protocol) =>
    provider.upstream_protocols.includes(protocol),
  );
}
