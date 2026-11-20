<script setup lang="ts">
import { Input, Select } from "@stellar/ui";
import type { AccumulateCycle, QuotaRule, ResetCycle } from "./rule";
import { accumulateOptions, minimumResetFor, resetOptionsFor } from "./rule";

defineOptions({ name: "ProviderQuotaRuleEditor" });

const props = defineProps<{
  modelValue: QuotaRule;
  disabled?: boolean;
}>();
const emit = defineEmits<{ "update:modelValue": [value: QuotaRule] }>();

const resetOptions = computed(() =>
  resetOptionsFor(props.modelValue.accumulate),
);

function update(patch: Partial<QuotaRule>) {
  emit("update:modelValue", { ...props.modelValue, ...patch });
}

function setAmount(value: string | number) {
  update({ amount: String(value) });
}

function setAccumulate(value: string | number | boolean) {
  const accumulate = String(value) as AccumulateCycle;
  const minimum = minimumResetFor(accumulate);
  const reset =
    resetOptionsFor(accumulate).find(
      (option) => option.value === props.modelValue.reset,
    )?.disabled === true
      ? minimum
      : props.modelValue.reset;
  update({ accumulate, reset });
}

function setReset(value: string | number | boolean) {
  update({ reset: String(value) as ResetCycle });
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
    <Select
      class="rule-editor__chip"
      :model-value="modelValue.accumulate"
      :options="accumulateOptions"
      :disabled="disabled"
      @update:model-value="setAccumulate"
    />
    ，
    <Select
      class="rule-editor__chip"
      :model-value="modelValue.reset"
      :options="resetOptions"
      :disabled="disabled"
      @update:model-value="setReset"
    />
  </div>
</template>

<style scoped>
.rule-editor {
  display: grid;
  width: 100%;
  grid-template-columns:
    auto minmax(120px, 1fr) auto minmax(120px, 1fr) auto
    minmax(120px, 1fr);
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--st-text-secondary);
  font-size: 13px;
  line-height: 1.5;
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
