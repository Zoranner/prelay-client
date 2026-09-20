<script setup lang="ts">
import {
  Alert,
  Button,
  EmptyState,
  FormField,
  Input,
  Select,
} from "@stellar/ui";
import type { CatalogProvider, Provider } from "~/stores/relay";
import {
  type ProviderFormPayload,
  type ProviderOperationInput,
  useProviderForm,
} from "~/composables/useProviderForm";
import type { ProviderOperationResult } from "~/utils/providerOperations";
import { modelCatalogLabel } from "~/utils/modelCatalog";
import ProviderModelToggles, {
  type ProviderModelToggle,
} from "~/components/providers/ProviderModelToggles.vue";

const props = defineProps<{
  provider?: Provider | null;
  catalogProviders: CatalogProvider[];
  pending?: boolean;
  canEdit?: boolean;
  canTest?: boolean;
  retired?: boolean;
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
      models: string[];
    },
  ];
  cancel: [];
  "dirty-change": [dirty: boolean];
}>();

const {
  apiKey,
  baseUrl,
  enabledModels,
  isModelEnabled,
  imageGenerationModelOptions,
  languageModelOptions,
  name,
  orderedUpstreamProtocols,
  protocolBaseUrls,
  protocolLabel,
  providerTemplate,
  providerTemplateOptions,
  removeRetiredModel,
  requestProtocolTest,
  retiredModels,
  retiredProvider,
  selectProviderTemplate,
  submit: createPayload,
  toggleModel,
} = useProviderForm({
  provider: () => props.provider,
  catalogProviders: () => props.catalogProviders,
  testProtocol: props.testProtocol,
  onDirtyChange: (dirty) => emit("dirty-change", dirty),
});

function submit() {
  if (props.canEdit === false) return;
  const payload = createPayload();
  if (payload) emit("save", payload);
}

function enabledCount(models: ProviderModelToggle[]) {
  // 已下架的模型还在供应商清单里，算在开启数里，由红色标签标出问题。
  const enabled = models.filter((model) => model.state !== "available").length;
  return enabled === models.length
    ? models.length
    : `${enabled} / ${models.length}`;
}

function toggleRows(
  options: { value: string; label: string }[],
): ProviderModelToggle[] {
  return options.map((option) => ({
    id: option.value,
    label: option.label,
    state: isModelEnabled(option.value) ? "enabled" : "available",
  }));
}

function retiredRows(modelIds: string[]): ProviderModelToggle[] {
  return modelIds.map((modelId) => ({
    id: modelId,
    label: modelCatalogLabel(modelId),
    state: "retired",
  }));
}

const languageToggleModels = computed(() =>
  toggleRows(languageModelOptions.value),
);
const imageToggleModels = computed(() =>
  toggleRows(imageGenerationModelOptions.value),
);
const retiredToggleModels = computed<ProviderModelToggle[]>(() =>
  retiredRows(retiredModels.value),
);
const retiredNotice = computed(() => props.retired || retiredProvider.value);
</script>

<template>
  <form id="provider-form" class="provider-form" @submit.prevent="submit">
    <Alert v-if="retiredNotice" semantic="warning" icon="ph:warning-circle">
      该供应商已下架：现有配置可在短期内继续使用，但配置不支持再修改，推荐删除该供应商并选择使用其他供应商。
    </Alert>
    <section class="form-section">
      <h3>连接配置</h3>
      <div class="form-fields">
        <Select
          v-model="providerTemplate"
          label="供应商"
          :options="providerTemplateOptions"
          :disabled="canEdit === false || pending"
          @change="selectProviderTemplate"
        />
        <Input
          v-model="name"
          label="名称"
          autocomplete="off"
          :disabled="canEdit === false || pending"
        />
        <FormField label="API Key">
          <Input
            v-model="apiKey"
            type="password"
            placeholder="填写上游 API Key"
            :disabled="canEdit === false || pending"
          />
        </FormField>
        <Input
          v-model="baseUrl"
          label="Base URL"
          type="url"
          :disabled="canEdit === false || pending"
        />
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
              :disabled="canEdit === false || pending"
            />
            <Button
              v-if="protocol !== 'images_generations'"
              square
              type="button"
              icon="ph:flask"
              aria-label="测试协议"
              title="测试协议"
              :disabled="canEdit === false || canTest === false || pending"
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
          <span>{{ enabledModels.length }} 个</span>
        </div>
      </div>
      <div class="model-list">
        <div
          v-if="languageToggleModels.length"
          class="model-group model-group--language"
        >
          <h4 class="model-group__header">
            <span>语言模型</span>
            <small>{{ enabledCount(languageToggleModels) }}</small>
          </h4>
          <ProviderModelToggles
            :models="languageToggleModels"
            :can-toggle="canEdit !== false && !pending"
            @toggle="toggleModel"
          />
        </div>
        <div
          v-if="imageToggleModels.length"
          class="model-group model-group--image"
        >
          <h4 class="model-group__header">
            <span>图像生成模型</span>
            <small>{{ enabledCount(imageToggleModels) }}</small>
          </h4>
          <ProviderModelToggles
            :models="imageToggleModels"
            :can-toggle="canEdit !== false && !pending"
            @toggle="toggleModel"
          />
        </div>
        <div
          v-if="retiredToggleModels.length"
          class="model-group model-group--retired"
        >
          <h4 class="model-group__header">
            <span>已下架</span>
            <small>{{ retiredToggleModels.length }}</small>
          </h4>
          <ProviderModelToggles
            :models="retiredToggleModels"
            :can-toggle="canEdit !== false && !pending"
            @remove="removeRetiredModel"
          />
        </div>
        <EmptyState
          v-if="
            !languageToggleModels.length &&
            !imageToggleModels.length &&
            !retiredToggleModels.length
          "
          size="small"
          title="该目录条目没有模型"
        />
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
.form-section h3 {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 15px;
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
</style>
