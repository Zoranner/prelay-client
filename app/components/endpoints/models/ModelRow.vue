<script setup lang="ts">
import { Button, Icon, Tag } from "@stellar/ui";
import {
  endpointMappingIssueHint,
  endpointMappingIssueLabel,
  type EndpointMappingIssue,
} from "~/utils/model/endpoints";

defineProps<{
  provider: string;
  shared?: boolean;
  model: string;
  issue?: EndpointMappingIssue | null;
  canMoveUp: boolean;
  canMoveDown: boolean;
}>();

const emit = defineEmits<{
  move: [delta: number];
  remove: [];
}>();
</script>

<template>
  <div class="model-row">
    <div
      class="model-row__label"
      :title="issue ? endpointMappingIssueHint(issue) : undefined"
    >
      <span class="model-row__name">
        <small>{{ provider }} / {{ model }}</small>
        <span
          v-if="shared"
          class="model-row__shared"
          aria-label="共享供应商"
          title="共享供应商"
        >
          <Icon icon="ph:share-network" size="14" />
        </span>
      </span>
      <Tag v-if="issue" semantic="error" size="small" icon="ph:warning-circle">
        {{ endpointMappingIssueLabel(issue) }}
      </Tag>
    </div>
    <div class="model-row__actions">
      <Button
        square
        size="small"
        variant="ghost"
        icon="ph:arrow-up"
        aria-label="上移供应商优先级"
        title="上移供应商优先级"
        :disabled="!canMoveUp"
        @click="emit('move', -1)"
      />
      <Button
        square
        size="small"
        variant="ghost"
        icon="ph:arrow-down"
        aria-label="下移供应商优先级"
        title="下移供应商优先级"
        :disabled="!canMoveDown"
        @click="emit('move', 1)"
      />
      <Button
        square
        size="small"
        semantic="error"
        variant="solid"
        icon="ph:trash"
        aria-label="删除供应商映射"
        title="删除供应商映射"
        @click="emit('remove')"
      />
    </div>
  </div>
</template>

<style scoped>
.model-row__label {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-sm);
}

.model-row__name {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-xs);
}

.model-row small {
  color: var(--st-text-secondary);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-row__shared {
  display: inline-flex;
  flex: 0 0 auto;
  color: var(--st-text-muted);
}

.model-row__actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
</style>
