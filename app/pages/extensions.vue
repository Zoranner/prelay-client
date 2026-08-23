<script setup lang="ts">
import { Button, List, ListItem, RadioGroup } from "stellar-ui";
import type {
  AgentClient,
  AgentExtensionKind,
  AgentExtensionsSnapshot,
} from "~/stores/relay";
import codexIcon from "@lobehub/icons-static-svg/icons/openai.svg";
import claudeIcon from "@lobehub/icons-static-svg/icons/claudecode-color.svg";
import ExtensionList from "~/components/extensions/ExtensionList.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

const { pending, error, invokeLocalCommand } = useLocalCommand();
const snapshot = ref<AgentExtensionsSnapshot>({ clients: [] });
const activeClient = ref<AgentClient | "">("");
const activeKind = ref<AgentExtensionKind>("mcp");
const kinds: AgentExtensionKind[] = ["plugin", "mcp", "skill"];
const activeClientExtensions = computed(() =>
  snapshot.value.clients.find((client) => client.client === activeClient.value),
);
const kindOptions = computed(() =>
  kinds.map((kind) => ({
    value: kind,
    label: `${kindLabel(kind)} ${
      activeClientExtensions.value
        ? extensionsFor(activeClientExtensions.value, kind).length
        : 0
    }`,
    icon: kindIcon(kind),
  })),
);

async function loadExtensions() {
  try {
    snapshot.value = await invokeLocalCommand<AgentExtensionsSnapshot>(
      "extensions_list",
    );
    if (!snapshot.value.clients.some((client) => client.client === activeClient.value)) {
      activeClient.value = snapshot.value.clients[0]?.client ?? "";
    }
  } catch {
    // The local command composable exposes the stable error to this view.
  }
}

function clientLabel(client: AgentClient) {
  return client === "codex" ? "Codex" : "Claude Code";
}

function clientIcon(client: AgentClient) {
  return client === "codex" ? codexIcon : claudeIcon;
}

function kindLabel(kind: AgentExtensionKind) {
  return { mcp: "MCP", skill: "Skill", plugin: "插件" }[kind];
}

function kindIcon(kind: AgentExtensionKind) {
  return {
    mcp: "ph:plugs-connected",
    skill: "ph:book-open-text",
    plugin: "ph:puzzle-piece",
  }[kind];
}

function extensionsFor(
  client: AgentExtensionsSnapshot["clients"][number],
  kind: AgentExtensionKind,
) {
  return client.extensions.filter((extension) => extension.kind === kind);
}

onMounted(loadExtensions);
</script>

<template>
  <main class="page-workbench">
    <PanelSection title="扩展">
      <template #header-actions>
        <Button
          square
          icon="ph:arrows-clockwise"
          :loading="pending"
          aria-label="刷新"
          title="刷新"
          @click="loadExtensions"
        />
      </template>
      <p v-if="error" class="extensions-error">{{ error.message }}</p>
      <div v-if="activeClientExtensions" class="extensions-content">
        <List class="agent-client-list" :divided="false">
          <ListItem
            v-for="client in snapshot.clients"
            :key="client.client"
            :active="activeClient === client.client"
            clickable
            :extra="String(client.extensions.length)"
            @click="activeClient = client.client"
          >
            <template #prefix>
              <img
                :src="clientIcon(client.client)"
                :alt="clientLabel(client.client)"
                class="agent-client-icon"
                :class="{
                  'agent-client-icon--monochrome': client.client === 'codex',
                }"
              />
            </template>
            {{ clientLabel(client.client) }}
          </ListItem>
        </List>
        <div class="extension-main">
          <RadioGroup v-model="activeKind" :options="kindOptions" variant="button" />
          <div class="extension-results">
            <ExtensionList
              :extensions="extensionsFor(activeClientExtensions, activeKind)"
            />
          </div>
        </div>
      </div>
      <p v-else-if="!error" class="extensions-empty">未检测到本机智能体扩展。</p>
    </PanelSection>
  </main>
</template>

<style scoped>
.page-workbench {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--pr-workbench-padding);
}

.extensions-content {
  display: grid;
  flex: 1;
  min-height: 0;
  grid-template-columns: 184px minmax(0, 1fr);
}

.agent-client-list {
  min-height: 0;
  padding-right: var(--spacing-md);
  border-right: 1px solid var(--st-border-divider);
}

.extension-main {
  display: flex;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  padding-left: var(--spacing-lg);
}

.agent-client-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.agent-client-icon--monochrome {
  filter: var(--pr-monochrome-icon-filter);
}

.extension-results {
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
  padding-top: var(--spacing-md);
  overflow: hidden;
}

.extensions-error {
  margin: 0 0 var(--spacing-md);
  color: var(--st-text-danger);
  font-family: var(--font-family-mono);
  font-size: 12px;
  overflow-wrap: anywhere;
}

.extensions-empty {
  margin: auto;
  color: var(--st-text-muted);
}
</style>
