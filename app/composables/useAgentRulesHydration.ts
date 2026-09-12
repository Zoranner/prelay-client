import type { AgentClient, AgentSettings } from "~/stores/relay";

type AgentRulesHydrationOptions = {
  activeClient: Readonly<Ref<AgentClient>>;
  settings: Readonly<Ref<Partial<Record<AgentClient, AgentSettings>>>>;
  hydrate: (client: AgentClient, active: boolean) => void;
};

export function useAgentRulesHydration(options: AgentRulesHydrationOptions) {
  watch(
    [
      () => options.settings.value.codexCli,
      () => options.settings.value.chatgpt,
      () => options.settings.value.openCode,
    ],
    ([codexCli, chatgpt, openCode]) => {
      for (const settings of [codexCli, chatgpt, openCode]) {
        if (!settings) continue;
        options.hydrate(
          settings.client,
          settings.client === options.activeClient.value,
        );
      }
    },
    { immediate: true },
  );
}
