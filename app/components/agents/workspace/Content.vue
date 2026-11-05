<script setup lang="ts">
defineOptions({ name: "AgentWorkspaceContent" });

import { Button, Icon, Loading, RadioGroup } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import type { AgentSection, AgentSectionOption } from "~/utils/agent/client";
import AgentHookList from "~/components/agents/workspace/items/HookList.vue";
import AgentMcpList from "~/components/agents/workspace/items/McpList.vue";
import AgentPluginList from "~/components/agents/workspace/items/PluginList.vue";
import AgentRulesEditor from "~/components/agents/workspace/RulesEditor.vue";
import AgentSkillList from "~/components/agents/workspace/items/SkillList.vue";
import WorkspaceLayout from "~/components/shell/layout/WorkspaceLayout.vue";

const activeSection = defineModel<AgentSection>("activeSection", {
  required: true,
});
const rules = defineModel<string>("rules", { required: true });
const props = defineProps<{
  clientContentLoading: boolean;
  clientInstalled: boolean;
  clientStatusesLoaded: boolean;
  itemPending: boolean;
  sectionItems: AgentItem[];
  sectionOptions: AgentSectionOption[];
}>();
const emit = defineEmits<{
  openSettings: [];
  uninstall: [item: AgentItem];
}>();

const ready = computed(
  () =>
    props.clientStatusesLoaded &&
    !props.clientContentLoading &&
    props.clientInstalled,
);

function updateActiveSection(value: string | number | boolean | null) {
  if (typeof value === "string") activeSection.value = value as AgentSection;
}
</script>

<template>
  <WorkspaceLayout :ready="ready">
    <template #status>
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
      <section v-else class="workspace-unavailable">
        <Icon icon="ph:download-simple" size="24" />
        <p>未检测到本机安装</p>
      </section>
    </template>
    <template #toolbar>
      <RadioGroup
        :model-value="activeSection"
        :options="sectionOptions"
        variant="button"
        @update:model-value="updateActiveSection"
      />
      <div class="workspace-toolbar__actions">
        <Button
          icon="ph:sliders-horizontal"
          aria-label="配置"
          title="配置"
          @click="emit('openSettings')"
        >
          配置
        </Button>
      </div>
    </template>
    <AgentRulesEditor v-if="activeSection === 'rules'" v-model="rules" />
    <AgentPluginList
      v-else-if="activeSection === 'plugin'"
      :items="sectionItems"
      :pending="itemPending"
      @uninstall="emit('uninstall', $event)"
    />
    <AgentMcpList
      v-else-if="activeSection === 'mcp'"
      :items="sectionItems"
      :pending="itemPending"
      @uninstall="emit('uninstall', $event)"
    />
    <AgentSkillList
      v-else-if="activeSection === 'skill'"
      :items="sectionItems"
      :pending="itemPending"
      @uninstall="emit('uninstall', $event)"
    />
    <AgentHookList
      v-else
      :items="sectionItems"
      :pending="itemPending"
      @uninstall="emit('uninstall', $event)"
    />
  </WorkspaceLayout>
</template>

<style scoped>
.workspace-unavailable {
  display: grid;
  justify-items: center;
  gap: var(--spacing-sm);
  color: var(--st-text-secondary);
}

.workspace-unavailable p {
  margin: 0;
}
</style>
