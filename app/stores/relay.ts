import type { BootstrapState } from "~/types/relay";

export function useRelayStore() {
  const bootstrap = useState<BootstrapState | null>(
    "relay-bootstrap",
    () => null,
  );

  function setBootstrap(value: BootstrapState) {
    bootstrap.value = value;
  }

  function clearBootstrap() {
    bootstrap.value = null;
  }

  return { bootstrap, setBootstrap, clearBootstrap };
}
