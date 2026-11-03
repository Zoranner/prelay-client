import { useNotification } from "@stellar/ui";
import type { AgentClient } from "~/types/relay";
import { clientSupportsRules } from "~/utils/agent/client";
import {
  codexSettingsPayload,
  type AgentConfiguration,
  type AgentSettingsSaveRequest,
} from "~/utils/agent/settings";

type AgentRulesOptions = {
  configuration: AgentConfiguration;
  reloadSettings: (client: AgentClient) => Promise<void>;
  save: (request: AgentSettingsSaveRequest) => Promise<unknown>;
};

export function useAgentRules(options: AgentRulesOptions) {
  const notifications = useNotification();
  const draft = reactive({
    codexCli: "",
    chatgpt: "",
  });
  const saving = ref(false);
  const dirty = computed(
    () =>
      draft.codexCli !== options.configuration.codexCli.rules ||
      draft.chatgpt !== options.configuration.chatgpt.rules,
  );
  let loaded = false;
  let suppressSave = false;
  let timer: ReturnType<typeof setTimeout> | undefined;

  function replace(client: AgentClient, rules: string) {
    suppressSave = true;
    draft[client] = rules;
    queueMicrotask(() => {
      suppressSave = false;
    });
  }

  function hydrate(client: AgentClient, active: boolean) {
    replace(client, options.configuration[client].rules);
    if (clientSupportsRules(client) && active) {
      void nextTick().then(() => {
        if (active) loaded = true;
      });
    }
  }

  async function save(client: AgentClient) {
    saving.value = true;
    try {
      const settings =
        client === "codexCli"
          ? codexSettingsPayload({
              ...options.configuration.codexCli,
              rules: draft.codexCli,
            })
          : codexSettingsPayload({
              ...options.configuration.chatgpt,
              rules: draft.chatgpt,
            });
      await options.save({ settings: { client, settings }, connection: null });
      options.configuration[client].rules = draft[client];
      notifications.success("规则已保存");
      void options.reloadSettings(client);
    } finally {
      saving.value = false;
    }
  }

  function schedule(client: AgentClient) {
    if (!loaded || suppressSave) return;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      void save(client);
    }, 500);
  }

  function discard() {
    if (timer) clearTimeout(timer);
    replace("codexCli", options.configuration.codexCli.rules);
    replace("chatgpt", options.configuration.chatgpt.rules);
  }

  function pause() {
    loaded = false;
  }

  function stop() {
    if (timer) clearTimeout(timer);
  }

  watch(
    () => draft.codexCli,
    () => schedule("codexCli"),
  );
  watch(
    () => draft.chatgpt,
    () => schedule("chatgpt"),
  );
  return { discard, dirty, draft, hydrate, pause, saving, stop };
}
