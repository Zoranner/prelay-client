<script setup lang="ts">
import { Button } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import {
  agentUninstallDisabled,
  agentUninstallLabel,
} from "~/utils/agent/itemDisplay";

const props = defineProps<{
  item: AgentItem;
  pending?: boolean;
}>();
const emit = defineEmits<{
  uninstall: [item: AgentItem];
}>();

const disabled = computed(() =>
  agentUninstallDisabled(props.item, Boolean(props.pending)),
);
const label = computed(() => agentUninstallLabel(props.item));
</script>

<template>
  <Button
    square
    size="small"
    semantic="error"
    variant="ghost"
    icon="ph:trash"
    :aria-label="label"
    :title="label"
    :disabled="disabled"
    @click.stop="emit('uninstall', item)"
  />
</template>
