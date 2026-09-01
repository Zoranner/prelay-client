<script setup lang="ts">
import { Button, Icon, Loading, RadioGroup } from "@stellar/ui";
import type {
  AgentClient,
  AgentItem,
  AgentItemKind,
  ExtensionCatalogKind,
  ExtensionCatalogPackage,
} from "~/stores/relay";
import AgentItemList from "~/components/agents/AgentItemList.vue";
import AgentRulesEditor from "~/components/agents/AgentRulesEditor.vue";
import ExtensionCatalogTable from "~/components/extensions/ExtensionCatalogTable.vue";

type AgentSection = "rules" | AgentItemKind;
type SectionOption<T extends string> = {
  icon: string;
  label: string;
  value: T;
};

const {
  activeExtensionSection,
  activeSection,
  clientContentLoading,
  clientInstalled,
  clientStatusesLoaded,
  extensionLoading,
  extensionPackages,
  extensionSectionOptions,
  itemPending,
  rules,
  sectionItems,
  sectionOptions,
  workspace,
} = defineProps<{
  activeExtensionSection: ExtensionCatalogKind;
  activeSection: AgentSection;
  clientContentLoading: boolean;
  clientInstalled: boolean;
  clientStatusesLoaded: boolean;
  extensionLoading: boolean;
  extensionPackages: ExtensionCatalogPackage[];
  extensionSectionOptions: SectionOption<ExtensionCatalogKind>[];
  itemPending: boolean;
  rules: string;
  sectionItems: AgentItem[];
  sectionOptions: SectionOption<AgentSection>[];
  workspace: AgentClient | "extensions";
}>();

const showItemStatus = computed(() => activeSection !== "skill");

const emit = defineEmits<{
  detail: [extension: ExtensionCatalogPackage];
  install: [extension: ExtensionCatalogPackage];
  openSettings: [];
  uninstall: [item: AgentItem];
  "update:activeExtensionSection": [section: ExtensionCatalogKind];
  "update:activeSection": [section: AgentSection];
  "update:rules": [rules: string];
}>();

function updateActiveSection(value: string | number | boolean | null) {
  if (typeof value === "string")
    emit("update:activeSection", value as AgentSection);
}

function updateExtensionSection(value: string | number | boolean | null) {
  if (typeof value === "string") {
    emit("update:activeExtensionSection", value as ExtensionCatalogKind);
  }
}
</script>

<template>
  <div :key="workspace" class="agent-main">
    <template v-if="workspace === 'extensions'">
      <div class="agent-toolbar">
        <RadioGroup
          :model-value="activeExtensionSection"
          :options="extensionSectionOptions"
          variant="button"
          @update:model-value="updateExtensionSection"
        />
      </div>
      <div class="item-results">
        <ExtensionCatalogTable
          :packages="extensionPackages"
          :pending="extensionLoading"
          @detail="emit('detail', $event)"
          @install="emit('install', $event)"
        />
      </div>
    </template>
    <template v-else>
      <Loading
        v-if="!clientStatusesLoaded"
        visible
        text="正在检测智能体安装状态..."
      />
      <Loading
        v-else-if="clientContentLoading"
        visible
        text="正在读取智能体设置..."
      />
      <section v-else-if="!clientInstalled" class="agent-unavailable">
        <Icon icon="ph:download-simple" size="24" />
        <p>未检测到本机安装</p>
      </section>
      <template v-else>
        <div class="agent-toolbar">
          <RadioGroup
            :model-value="activeSection"
            :options="sectionOptions"
            variant="button"
            @update:model-value="updateActiveSection"
          />
          <div class="agent-toolbar__actions">
            <Button
              icon="ph:sliders-horizontal"
              aria-label="配置"
              title="配置"
              @click="emit('openSettings')"
            >
              配置
            </Button>
          </div>
        </div>
        <AgentRulesEditor
          v-if="activeSection === 'rules'"
          :model-value="rules"
          @update:model-value="emit('update:rules', $event)"
        />
        <div v-else class="item-results">
          <AgentItemList
            :items="sectionItems"
            :pending="itemPending"
            :show-status="showItemStatus"
            @uninstall="emit('uninstall', $event)"
          />
        </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.agent-main {
  display: flex;
  position: relative;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  padding-left: var(--spacing-lg);
}

.agent-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 0 0 var(--spacing-md);
}

.item-results {
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
  padding-top: var(--spacing-md);
  overflow: hidden;
}

.agent-unavailable {
  display: grid;
  flex: 1;
  place-content: center;
  justify-items: center;
  gap: var(--spacing-sm);
  color: var(--st-text-secondary);
}

.agent-unavailable p {
  margin: 0;
}

.agent-toolbar__actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
</style>
