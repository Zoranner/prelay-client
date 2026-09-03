<script setup lang="ts">
import { Button, FormField, Input, Select, Tag } from "@stellar/ui";
import type { CatalogProvider, Provider } from "~/stores/relay";
import {
  type ProviderFormPayload,
  type ProviderOperationInput,
  useProviderForm,
} from "~/composables/useProviderForm";
import type { ProviderOperationResult } from "~/utils/providerOperations";

const props = defineProps<{
  provider?: Provider | null;
  catalogProviders: CatalogProvider[];
  pending?: boolean;
  testProtocol: (
    input: ProviderOperationInput,
  ) => Promise<ProviderOperationResult>;
}>();

const emit = defineEmits<{
  save: [
    payload: {
      id?: string;
      name: string;
      provider_type: string;
      base_url: string;
      api_key: string;
      capabilities: ProviderFormPayload["capabilities"];
      models: ProviderFormPayload["models"];
    },
  ];
  cancel: [];
  "dirty-change": [dirty: boolean];
}>();

const {
  apiKey,
  baseUrl,
  imageGenerationModelOptions,
  languageModelOptions,
  models,
  name,
  orderedUpstreamProtocols,
  protocolBaseUrls,
  protocolLabel,
  providerTemplate,
  providerTemplateOptions,
  requestProtocolTest,
  selectProviderTemplate,
  submit: createPayload,
} = useProviderForm({
  provider: () => props.provider,
  catalogProviders: () => props.catalogProviders,
  testProtocol: props.testProtocol,
  onDirtyChange: (dirty) => emit("dirty-change", dirty),
});

function submit() {
  const payload = createPayload();
  if (payload) emit("save", payload);
}
</script>

<template>
  <form id="provider-form" class="provider-form" @submit.prevent="submit">
    <section class="form-section">
      <h3>连接配置</h3>
      <div class="form-fields">
        <Select
          v-model="providerTemplate"
          label="供应商"
          :options="providerTemplateOptions"
          @change="selectProviderTemplate"
        />
        <Input v-model="name" label="名称" autocomplete="off" />
        <FormField label="API Key">
          <Input
            v-model="apiKey"
            type="password"
            placeholder="填写上游 API Key"
          />
        </FormField>
        <Input v-model="baseUrl" label="Base URL" type="url" />
        <div class="protocol-urls" aria-label="支持协议">
          <div
            v-for="protocol in orderedUpstreamProtocols"
            :key="protocol"
            class="protocol-url-row"
          >
            <span class="protocol-label">{{ protocolLabel(protocol) }}</span>
            <Input
              v-model="protocolBaseUrls[protocol]"
              :placeholder="baseUrl || '填写协议地址'"
            />
            <Button
              v-if="protocol !== 'images_generations'"
              square
              type="button"
              icon="ph:flask"
              aria-label="测试协议"
              title="测试协议"
              :disabled="pending"
              @click="requestProtocolTest(protocol)"
            />
          </div>
        </div>
      </div>
    </section>

    <section class="form-section">
      <div class="section-header">
        <h3>模型清单</h3>
        <div class="section-header__actions">
          <span>{{ models.length }} 个</span>
        </div>
      </div>
      <div class="model-list">
        <div
          v-if="languageModelOptions.length"
          class="model-group model-group--language"
        >
          <h4 class="model-group__header">
            <span>语言模型</span>
            <small>{{ languageModelOptions.length }}</small>
          </h4>
          <div class="model-tags">
            <Tag
              v-for="modelOption in languageModelOptions"
              :key="`language-${modelOption.value}`"
              class="model-tag model-tag--language"
              size="small"
              variant="primary"
            >
              {{ modelOption.label }}
            </Tag>
          </div>
        </div>
        <div
          v-if="imageGenerationModelOptions.length"
          class="model-group model-group--image"
        >
          <h4 class="model-group__header">
            <span>图像生成模型</span>
            <small>{{ imageGenerationModelOptions.length }}</small>
          </h4>
          <div class="model-tags">
            <Tag
              v-for="modelOption in imageGenerationModelOptions"
              :key="`image-${modelOption.value}`"
              class="model-tag model-tag--image"
              size="small"
              variant="warning"
            >
              {{ modelOption.label }}
            </Tag>
          </div>
        </div>
        <p v-if="!models.length" class="empty-text">暂无模型。</p>
      </div>
    </section>
  </form>
</template>

<style scoped>
.provider-form,
.form-section,
.form-fields,
.model-list {
  display: grid;
  gap: var(--spacing-lg);
}
.provider-form {
  padding: var(--spacing-lg);
}
.form-section {
  margin: 0;
}
.form-section h3,
.form-section p {
  margin: 0;
}
.form-section h3 {
  color: var(--st-text-primary);
  font-size: 15px;
}
.form-section p,
.empty-text {
  color: var(--st-text-secondary);
}
.protocol-url-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
.protocol-urls {
  display: grid;
  gap: var(--spacing-sm);
}
.protocol-url-row > :nth-child(2) {
  flex: 1;
}
.protocol-label {
  width: 160px;
  flex: 0 0 160px;
  color: var(--st-text-secondary);
  white-space: nowrap;
}
@media (max-width: 720px) {
  .protocol-url-row {
    align-items: stretch;
    flex-direction: column;
  }
  .protocol-label {
    width: auto;
    flex-basis: auto;
  }
}
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--spacing-md);
}
.section-header__actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
.section-header__actions > span {
  color: var(--st-text-secondary);
}
.model-list {
  grid-template-columns: minmax(0, 1fr);
  gap: var(--spacing-sm);
}
.model-group {
  display: grid;
  gap: var(--spacing-sm);
  min-width: 0;
  align-content: start;
}
.model-group:only-child {
  grid-column: 1 / -1;
}
.model-group + .model-group {
  margin-top: var(--spacing-md);
}
.model-group h4 {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
  margin: 0 0 var(--spacing-xs);
  color: var(--st-text-secondary);
  font-size: 13px;
  font-weight: 600;
}
.model-group--language h4 {
  color: var(--st-primary);
}
.model-group--image h4 {
  color: var(--st-warning);
}
.model-group h4 small {
  color: var(--st-text-muted);
  font-family: var(--font-family-mono);
  font-size: 12px;
  font-weight: 500;
}
.model-tags {
  display: flex;
  min-width: 0;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}
.model-tag {
  max-width: 100%;
  white-space: normal;
  overflow-wrap: anywhere;
}
.empty-text {
  grid-column: 1 / -1;
}
</style>
