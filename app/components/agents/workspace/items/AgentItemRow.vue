<script setup lang="ts">
import { Icon, Tooltip } from "@stellar/ui";
import type { AgentItem } from "~/types/relay";
import ItemError from "~/components/agents/workspace/items/ItemError.vue";
import ItemSource from "~/components/agents/workspace/items/ItemSource.vue";
import ItemUninstall from "~/components/agents/workspace/items/ItemUninstall.vue";

defineProps<{
  item: AgentItem;
  primary: string;
  icon: string;
  pending?: boolean;
}>();
const emit = defineEmits<{
  uninstall: [item: AgentItem];
}>();
</script>

<template>
  <article class="agent-item-row">
    <div class="agent-item-row__top">
      <span
        class="agent-item-row__icon"
        :class="`agent-item-row__icon--${item.kind}`"
      >
        <Icon :icon="icon" size="20" />
      </span>
      <div class="agent-item-row__summary">
        <span class="agent-item-row__identity">
          <span class="agent-item-row__primary" :title="primary">
            {{ primary }}
          </span>
          <ItemError :message="item.errorMessage" />
          <Tooltip :content="item.sourcePath" position="top">
            <span
              class="agent-item-row__path-trigger"
              tabindex="0"
              role="img"
              :aria-label="`来源路径：${item.sourcePath}`"
            >
              <svg
                class="agent-item-row__path-icon"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path
                  d="M3 6.5A1.5 1.5 0 0 1 4.5 5H9l2 2h8.5A1.5 1.5 0 0 1 21 8.5v9A1.5 1.5 0 0 1 19.5 19h-15A1.5 1.5 0 0 1 3 17.5v-11Z"
                />
                <path d="M3 10h18" />
              </svg>
            </span>
          </Tooltip>
        </span>
        <span class="agent-item-row__meta">
          <ItemSource :item="item" />
          <template v-if="$slots.facts">
            <span class="agent-item-row__separator">·</span>
            <slot name="facts" />
          </template>
        </span>
      </div>
    </div>
    <div class="agent-item-row__actions">
      <ItemUninstall
        :item="item"
        :pending="pending"
        @uninstall="emit('uninstall', $event)"
      />
    </div>
    <div v-if="$slots.detail" class="agent-item-row__detail">
      <slot name="detail" />
    </div>
  </article>
</template>

<style scoped>
.agent-item-row {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: start;
  row-gap: var(--spacing-sm);
  padding: var(--spacing-md) var(--spacing-md) var(--spacing-md)
    var(--spacing-lg);
  border-bottom: 1px solid var(--st-border-divider);
}

.agent-item-row:last-child {
  border-bottom: 0;
}

.agent-item-row__top {
  display: grid;
  min-width: 0;
  grid-template-columns: 45px minmax(0, 1fr);
  align-items: start;
  gap: var(--spacing-md);
}

.agent-item-row__icon {
  display: grid;
  width: 45px;
  height: 45px;
  place-items: center;
  border: 1px solid var(--st-border-divider);
  border-radius: var(--radius-md);
  background: var(--st-bg-surface);
  color: var(--st-text-secondary);
}

.agent-item-row__icon--plugin {
  color: var(--st-primary);
}

.agent-item-row__summary {
  display: grid;
  min-width: 0;
  min-height: 42px;
  align-content: center;
  gap: var(--spacing-xs);
}

.agent-item-row__identity,
.agent-item-row__meta {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-xs);
}

.agent-item-row__identity {
  line-height: 1.2;
}

.agent-item-row__primary {
  flex: 0 1 auto;
  min-width: 0;
  overflow: hidden;
  color: var(--st-text-primary);
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agent-item-row__path-trigger {
  display: inline-flex;
  width: 16px;
  height: 16px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  color: var(--st-text-muted);
  line-height: 1;
  cursor: help;
}

.agent-item-row__path-trigger:hover {
  color: var(--st-text-secondary);
}

.agent-item-row__path-icon {
  display: block;
  position: relative;
  top: 1px;
  width: 14px;
  height: 14px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.8;
}

.agent-item-row__meta {
  flex-wrap: wrap;
  color: var(--st-text-muted);
  font-size: 12px;
}

.agent-item-row__separator {
  color: var(--st-text-muted);
}

.agent-item-row__actions {
  display: flex;
  position: absolute;
  z-index: 1;
  top: var(--spacing-md);
  right: var(--spacing-md);
  align-items: flex-start;
  padding-left: var(--spacing-sm);
  background: var(--st-bg-elevated);
  opacity: 0;
  pointer-events: none;
  transition: opacity 120ms ease;
}

.agent-item-row:hover .agent-item-row__actions,
.agent-item-row:focus-within .agent-item-row__actions {
  opacity: 1;
  pointer-events: auto;
}

@media (hover: none) {
  .agent-item-row__actions {
    opacity: 1;
    pointer-events: auto;
  }
}

.agent-item-row__detail {
  display: grid;
  min-width: 0;
  grid-column: 1;
  gap: var(--spacing-sm);
  padding-left: calc(45px + var(--spacing-md));
}
</style>
