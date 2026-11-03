<script setup lang="ts">
defineOptions({ name: "ExtensionLibraryContent" });

import { Button, RadioGroup } from "@stellar/ui";
import type {
  ExtensionCatalogKind,
  ExtensionCatalogPackage,
} from "~/types/relay";
import ExtensionCatalogTable from "~/components/extensions/library/CatalogTable.vue";
import WorkspaceLayout from "~/components/shell/layout/WorkspaceLayout.vue";

const activeSection = defineModel<ExtensionCatalogKind>("activeSection", {
  required: true,
});
defineProps<{
  loading: boolean;
  packages: ExtensionCatalogPackage[];
  updates: number;
  updating: boolean;
}>();
const emit = defineEmits<{
  detail: [extension: ExtensionCatalogPackage];
  install: [extension: ExtensionCatalogPackage];
  updateAll: [];
}>();

const sectionOptions: Array<{
  value: ExtensionCatalogKind;
  label: string;
  icon: string;
}> = [
  { value: "rule", label: "规则", icon: "ph:notebook" },
  { value: "plugin", label: "插件", icon: "ph:puzzle-piece" },
];

function updateSection(value: string | number | boolean | null) {
  if (typeof value === "string") {
    activeSection.value = value as ExtensionCatalogKind;
  }
}
</script>

<template>
  <WorkspaceLayout>
    <template #toolbar>
      <RadioGroup
        :model-value="activeSection"
        :options="sectionOptions"
        variant="button"
        @update:model-value="updateSection"
      />
      <div class="workspace-toolbar__actions">
        <template v-if="updates > 0">
          <span class="workspace-update-hint">可更新</span>
          <Button
            icon="ph:arrow-circle-down"
            :disabled="updating"
            @click="emit('updateAll')"
          >
            {{ updating ? "更新中..." : "全部更新" }}
          </Button>
        </template>
      </div>
    </template>
    <ExtensionCatalogTable
      :packages="packages"
      :pending="loading"
      @detail="emit('detail', $event)"
      @install="emit('install', $event)"
    />
  </WorkspaceLayout>
</template>
