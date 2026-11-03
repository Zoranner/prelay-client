import type { BootstrapState, RelayEndpoint } from "~/types/relay";
import { useRelayStore } from "~/stores/relay";

export function useAgentPageData() {
  const { bootstrap, setBootstrap } = useRelayStore();
  const { pending, invokeCommand } = useRelayCommand();
  const endpoints = ref<RelayEndpoint[]>([]);

  async function load() {
    const endpointsRequest = invokeCommand<RelayEndpoint[]>("endpoints_list")
      .then((value) => {
        endpoints.value = value;
      })
      .catch(() => {
        // The application-level management API status owns endpoint failures.
      });
    const bootstrapRequest = bootstrap.value
      ? Promise.resolve()
      : invokeCommand<BootstrapState>("bootstrap")
          .then(setBootstrap)
          .catch(() => {
            // The application-level management API status owns bootstrap failures.
          });
    await Promise.all([endpointsRequest, bootstrapRequest]);
  }

  return { bootstrap, endpoints, load, pending };
}
