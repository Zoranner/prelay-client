<script setup lang="ts">
import { Input, Select } from "@stellar/ui";
import type { AccumulateCycle, QuotaRule } from "./rule";
import { accumulateOptions } from "./rule";

defineOptions({ name: "ProviderQuotaRuleEditor" });

const props = defineProps<{
  modelValue: QuotaRule;
  disabled?: boolean;
}>();
const emit = defineEmits<{ "update:modelValue": [value: QuotaRule] }>();

function update(patch: Partial<QuotaRule>) {
  emit("update:modelValue", { ...props.modelValue, ...patch });
}

function setAmount(value: string | number) {
  update({ amount: String(value) });
}

function setAccumulate(value: string | number | boolean) {
  update({ accumulate: String(value) as AccumulateCycle });
}
</script>

<template>
  <div class="rule-editor" :class="{ 'rule-editor--disabled': disabled }">
    每天
    <Input
      class="rule-editor__amount"
      :model-value="modelValue.amount"
      type="number"
      :step="0.01"
      placeholder="0"
      :disabled="disabled"
      @update:model-value="setAmount"
    />
    <span class="rule-editor__unit">亿</span>
    <span class="rule-editor__unit">，</span>
    <Select
      class="rule-editor__chip"
      :model-value="modelValue.accumulate"
      :options="accumulateOptions"
      :disabled="disabled"
      @update:model-value="setAccumulate"
    />
  </div>
</template>

<style scoped>
.rule-editor {
  display: grid;
  width: 100%;
  grid-template-columns: auto minmax(96px, 1fr) auto auto minmax(96px, 1fr);
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--st-text-secondary);
  font-size: 13px;
  line-height: 1.5;
  white-space: nowrap;
}

.rule-editor--disabled {
  opacity: 0.6;
}

.rule-editor__amount {
  width: 100%;
  min-width: 0;
}

.rule-editor__unit {
  color: var(--st-text-secondary);
}

.rule-editor__chip {
  width: 100%;
  min-width: 0;
}

@media (max-width: 720px) {
  .rule-editor {
    display: flex;
    flex-wrap: wrap;
  }

  .rule-editor__amount {
    width: 92px;
  }

  .rule-editor__chip {
    width: auto;
    flex: 1 1 120px;
  }
}
</style>
