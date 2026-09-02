<script setup lang="ts">
import { Button, FormField, Input, Popover, Select } from "@stellar/ui";
import type { Provider } from "~/stores/relay";
import {
  type ProviderFormPayload,
  type ProviderOperationInput,
  useProviderForm,
} from "~/composables/useProviderForm";
import type { ProviderOperationResult } from "~/utils/providerOperations";

const props = defineProps<{
  provider?: Provider | null;
  pending?: boolean;
  discoverModels: (
    input: ProviderOperationInput,
  ) => Promise<ProviderOperationResult>;
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
  addModel,
  apiKey,
  baseUrl,
  modelDraft,
  models,
  name,
  orderedUpstreamProtocols,
  protocolBaseUrls,
  protocolLabel,
  protocolOptions,
  providerTemplate,
  providerTemplateOptions,
  removeModel,
  requestDiscovery,
  requestProtocolTest,
  selectProviderTemplate,
  setModelPopover,
  showAddModel,
  submit: createPayload,
  upstreamProtocols,
} = useProviderForm({
  provider: () => props.provider,
  discoverModels: props.discoverModels,
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
          <Input v-model="apiKey" type="password" placeholder="填写上游 API Key" />
        </FormField>
        <Input v-model="baseUrl" label="Base URL" type="url" />
        <Select
          v-model="upstreamProtocols"
          multiple
          label="支持协议"
          placeholder="选择支持协议"
          :options="protocolOptions"
        />
        <div class="protocol-urls">
          <div
            v-for="protocol in orderedUpstreamProtocols"
            :key="protocol"
            class="protocol-url-row"
          >
            <span>{{ protocolLabel(protocol) }}</span>
            <Input
              v-model="protocolBaseUrls[protocol]"
              :placeholder="baseUrl || '填写协议地址'"
            />
            <Button
              v-if="protocol !== 'images_generations'"
              square
              type="button"
              size="small"
              icon="ph:plugs-connected"
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
          <Button
            type="button"
            size="small"
            icon="ph:download-simple"
            aria-label="获取模型"
            title="获取模型"
            :disabled="pending"
            @click="requestDiscovery"
            >获取</Button
          >
          <Popover
            v-model="showAddModel"
            position="bottom"
            align="right"
            size="large"
            @update:model-value="setModelPopover"
          >
            <Button size="small" type="button" variant="primary" icon="ph:plus">
              新增
            </Button>
            <template #title>新增模型</template>
            <template #content>
              <div class="model-popover">
                <Input
                  v-model="modelDraft"
                  label="上游模型"
                  placeholder="kimi-k2-0711-preview"
                  @enter="addModel"
                />
              </div>
            </template>
            <template #footer>
              <Button
                variant="primary"
                type="button"
                :disabled="pending"
                @click="addModel"
                >确认</Button
              >
            </template>
          </Popover>
        </div>
      </div>
      <div class="model-list">
        <div v-for="model in models" :key="model" class="model-row">
          <code>{{ model }}</code>
          <Button
            square
            size="small"
            variant="danger"
            icon="ph:trash"
            aria-label="删除模型"
            title="删除模型"
            @click="removeModel(model)"
          />
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
.protocol-url-row > span {
  width: 160px;
  color: var(--st-text-secondary);
  white-space: nowrap;
}
.protocol-url-row > :nth-child(2) {
  flex: 1;
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
  gap: var(--spacing-sm);
}
.model-row {
  display: flex;
  min-width: 0;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
  border-top: 1px solid var(--st-border-divider);
  padding: var(--spacing-sm) 0 0 var(--spacing-md);
}
.model-row code {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.model-popover {
  display: grid;
  gap: var(--spacing-md);
}
@media (max-width: 720px) {
  .protocol-url-row {
    align-items: stretch;
    flex-direction: column;
  }
  .protocol-url-row > span {
    width: auto;
  }
}
</style>
