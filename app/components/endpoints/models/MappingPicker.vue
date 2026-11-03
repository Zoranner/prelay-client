<script setup lang="ts">
import { Button, Popover, Select } from "@stellar/ui";

type ProviderOption = { label: string; value: string; read_only?: boolean };
type UpstreamOption = { label: string; value: string };

defineProps<{
  title: string;
  open: boolean;
  providerId: string;
  upstreamModel: string;
  providerOptions: ProviderOption[];
  upstreamOptions: UpstreamOption[];
  upstreamDisabled: boolean;
  pending?: boolean;
}>();

const emit = defineEmits<{
  "update:open": [open: boolean];
  "update:providerId": [value: string];
  "update:upstreamModel": [value: string];
  selectProvider: [];
  confirm: [];
}>();
</script>

<template>
  <Popover
    :model-value="open"
    position="auto"
    align="right"
    size="large"
    @update:model-value="emit('update:open', $event)"
  >
    <slot />
    <template #title>{{ title }}</template>
    <template #content>
      <div class="model-popover">
        <Select
          :model-value="providerId"
          label="供应商"
          :options="providerOptions"
          @update:model-value="emit('update:providerId', $event)"
          @change="emit('selectProvider')"
        />
        <Select
          :model-value="upstreamModel"
          label="上游模型"
          :disabled="upstreamDisabled"
          :options="upstreamOptions"
          @update:model-value="emit('update:upstreamModel', $event)"
        />
      </div>
    </template>
    <template #footer>
      <Button
        semantic="primary"
        variant="solid"
        type="button"
        :disabled="pending"
        @click="emit('confirm')"
      >
        确认
      </Button>
    </template>
  </Popover>
</template>

<style scoped>
.model-popover {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-md);
}
@media (max-width: 640px) {
  .model-popover {
    grid-template-columns: 1fr;
  }
}
</style>
