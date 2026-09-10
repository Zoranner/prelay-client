<script setup lang="ts">
import { Button, Input, Popover, Select, useNotification } from "@stellar/ui";
import type {
  EndpointModel,
  ProviderListItem,
  RelayEndpoint,
} from "~/stores/relay";
import {
  availableEndpointModelsForProvider,
  checkEndpointMapping,
  endpointModelsForProvider,
  groupEndpointModels,
  moveEndpointMapping,
  providerOptionLabel,
  type EndpointModelGroup,
} from "~/utils/endpointModels";
import {
  modelCatalogLabel,
  modelCatalogProviderModels,
} from "~/utils/modelCatalog";
import EndpointModelRow from "~/components/endpoints/EndpointModelRow.vue";
const props = defineProps<{
  endpoint?: RelayEndpoint | null;
  providers: ProviderListItem[];
  pending?: boolean;
}>();
const emit = defineEmits<{
  save: [
    payload: {
      id?: string;
      name: string;
      protocol: string;
      models: Array<Pick<EndpointModel, "provider_id" | "upstream_model">>;
    },
  ];
  cancel: [];
  "dirty-change": [dirty: boolean];
}>();
type ModelForm = {
  provider_id: string;
  upstream_model: string;
};
type EndpointModelDraft = Omit<EndpointModel, "model_name"> &
  Pick<EndpointModel, "model_name">;
const name = ref("");
const protocol = ref("openai");
const models = ref<EndpointModelDraft[]>([]);
const newModelForm = ref<ModelForm>(emptyModelForm());
const newProviderForm = ref<ModelForm>(emptyModelForm());
const showAddModel = ref(false);
const activeProviderGroup = ref<string | null>(null);
const notifications = useNotification();
const availableProviders = computed(() =>
  props.providers.filter(
    (provider) => modelCatalogProviderModels(provider.provider_type).length > 0,
  ),
);
const modelGroups = computed(() => groupEndpointModels(models.value));
const providerOptions = computed(() => [
  { label: "选择供应商", value: "" },
  ...availableProviders.value.map((provider) => ({
    ...providerOptionLabel(provider),
  })),
]);
let initialDraft = "";
function serializeDraft() {
  return JSON.stringify({
    name: name.value,
    protocol: protocol.value,
    models: models.value,
  });
}
watch(
  () => props.endpoint,
  (current) => {
    name.value = current?.name ?? "";
    protocol.value = current?.protocol ?? "openai";
    models.value = current?.models.map((model) => ({ ...model })) ?? [];
    newModelForm.value = emptyModelForm();
    newProviderForm.value = emptyModelForm();
    showAddModel.value = false;
    activeProviderGroup.value = null;
    initialDraft = serializeDraft();
    emit("dirty-change", false);
  },
  { immediate: true },
);
watch(serializeDraft, (draft) => emit("dirty-change", draft !== initialDraft));
function emptyModelForm(): ModelForm {
  return { provider_id: "", upstream_model: "" };
}
function modelsForProvider(providerId: string) {
  const provider = availableProviders.value.find(({ id }) => id === providerId);
  return provider ? endpointModelsForProvider(provider) : [];
}

function availableUpstreamModels(
  providerId: string,
  group?: EndpointModelGroup,
) {
  return availableEndpointModelsForProvider(
    modelsForProvider(providerId),
    models.value,
    providerId,
    group?.name,
  );
}

function providerForModel(model: Pick<EndpointModel, "provider_id">) {
  return props.providers.find((provider) => provider.id === model.provider_id);
}

function providerSourceLabel(provider: ProviderListItem | undefined) {
  if (!provider || provider.can_manage !== false) return "";
  return `共享自 ${provider.owner_display_name || "其他身份"} · 只读`;
}

function upstreamModelOptions(providerId: string, group?: EndpointModelGroup) {
  const availableModels = availableUpstreamModels(providerId, group);
  return [
    {
      label: providerId
        ? availableModels.length
          ? "选择上游模型"
          : "无可用模型"
        : "先选择供应商",
      value: "",
    },
    ...availableModels.map((model) => ({
      label: model.display_name?.trim() || modelCatalogLabel(model.model_name),
      value: model.model_name,
    })),
  ];
}

function selectProvider(form: ModelForm, group?: EndpointModelGroup) {
  form.upstream_model =
    availableUpstreamModels(form.provider_id, group)[0]?.model_name ?? "";
}

function addMapping(form: ModelForm, fixedModelName?: string) {
  const upstreamModel = form.upstream_model.trim();
  const failure = checkEndpointMapping({
    providerId: form.provider_id,
    upstreamModel,
    providerModels: modelsForProvider(form.provider_id),
    endpointModels: models.value,
    fixedModelName,
  });
  if (failure) {
    notifications.error(failure.message, { title: failure.title });
    return false;
  }
  models.value.push({
    model_name: fixedModelName ?? upstreamModel,
    provider_id: form.provider_id,
    upstream_model: upstreamModel,
  });
  return true;
}

function addModel() {
  if (addMapping(newModelForm.value)) {
    newModelForm.value = emptyModelForm();
    showAddModel.value = false;
  }
}

function addProvider(groupName: string) {
  if (addMapping(newProviderForm.value, groupName)) {
    newProviderForm.value = emptyModelForm();
    activeProviderGroup.value = null;
  }
}

function setModelPopover(open: boolean) {
  showAddModel.value = open;
  activeProviderGroup.value = null;
  if (!open) newModelForm.value = emptyModelForm();
}

function setProviderPopover(groupName: string, open: boolean) {
  activeProviderGroup.value = open ? groupName : null;
  showAddModel.value = false;
  newModelForm.value = emptyModelForm();
  newProviderForm.value = emptyModelForm();
}

function removeModel(index: number) {
  models.value.splice(index, 1);
}

function mappingPosition(group: EndpointModelGroup, index: number) {
  return group.mappings.findIndex((mapping) => mapping.index === index);
}

function moveMapping(index: number, delta: number) {
  models.value = moveEndpointMapping(models.value, index, delta);
}

function submit() {
  if (!name.value.trim()) {
    notifications.error("请填写接入点名称。", { title: "接入点配置不完整" });
    return;
  }
  if (!models.value.length) {
    notifications.error("请至少新增一个模型。", { title: "接入点配置不完整" });
    return;
  }
  emit("save", {
    ...(props.endpoint ? { id: props.endpoint.id } : {}),
    name: name.value.trim(),
    protocol: protocol.value,
    models: models.value.map(({ provider_id, upstream_model }) => ({
      provider_id,
      upstream_model,
    })),
  });
}
</script>

<template>
  <form id="endpoint-form" class="endpoint-form" @submit.prevent="submit">
    <section class="form-section">
      <h3>接入点配置</h3>
      <Input v-model="name" label="名称" placeholder="Codex" />
    </section>
    <section class="form-section">
      <div class="section-header">
        <h3>模型列表</h3>
        <div class="section-header__actions">
          <span>{{ modelGroups.length }} 个</span>
          <Popover
            v-model="showAddModel"
            position="auto"
            align="right"
            size="large"
            @update:model-value="setModelPopover"
          >
            <Button
              size="small"
              type="button"
              semantic="primary"
              variant="solid"
              icon="ph:plus"
              >新增</Button
            >
            <template #title>新增模型</template>
            <template #content>
              <div class="model-popover">
                <Select
                  v-model="newModelForm.provider_id"
                  label="供应商"
                  :options="providerOptions"
                  @change="selectProvider(newModelForm)"
                />
                <Select
                  v-model="newModelForm.upstream_model"
                  label="上游模型"
                  :disabled="
                    !modelsForProvider(newModelForm.provider_id).length
                  "
                  :options="upstreamModelOptions(newModelForm.provider_id)"
                />
              </div>
            </template>
            <template #footer>
              <Button
                semantic="primary"
                variant="solid"
                type="button"
                :disabled="pending"
                @click="addModel"
              >
                确认
              </Button>
            </template>
          </Popover>
        </div>
      </div>
      <div class="model-list">
        <div v-for="group in modelGroups" :key="group.name" class="model-group">
          <div class="model-group__header">
            <code :title="group.name">{{ group.displayName }}</code>
            <div class="model-group__actions">
              <small>{{ group.mappings.length }} 个供应商</small>
              <Popover
                :model-value="activeProviderGroup === group.name"
                position="auto"
                align="right"
                size="large"
                @update:model-value="setProviderPopover(group.name, $event)"
              >
                <Button size="small" type="button" icon="ph:plus">
                  新增
                </Button>
                <template #title>新增供应商</template>
                <template #content>
                  <div class="model-popover">
                    <Select
                      v-model="newProviderForm.provider_id"
                      label="供应商"
                      :options="providerOptions"
                      @change="selectProvider(newProviderForm, group)"
                    />
                    <Select
                      v-model="newProviderForm.upstream_model"
                      label="上游模型"
                      :disabled="
                        !availableUpstreamModels(
                          newProviderForm.provider_id,
                          group,
                        ).length
                      "
                      :options="
                        upstreamModelOptions(newProviderForm.provider_id, group)
                      "
                    />
                  </div>
                </template>
                <template #footer>
                  <Button
                    semantic="primary"
                    variant="solid"
                    type="button"
                    :disabled="pending"
                    @click="addProvider(group.name)"
                  >
                    确认
                  </Button>
                </template>
              </Popover>
            </div>
          </div>
          <EndpointModelRow
            v-for="mapping in group.mappings"
            :key="
              mapping.model.id ??
              `${mapping.model.provider_id}-${mapping.model.upstream_model}-${mapping.index}`
            "
            :provider="providerForModel(mapping.model)?.name ?? '已删除供应商'"
            :source="providerSourceLabel(providerForModel(mapping.model))"
            :model="
              mapping.model.display_name?.trim() ||
              modelCatalogLabel(
                mapping.model.model_name || mapping.model.upstream_model,
              )
            "
            :can-move-up="mappingPosition(group, mapping.index) > 0"
            :can-move-down="
              mappingPosition(group, mapping.index) < group.mappings.length - 1
            "
            @move="moveMapping(mapping.index, $event)"
            @remove="removeModel(mapping.index)"
          />
        </div>
        <p v-if="!models.length" class="empty-text">暂无模型。</p>
      </div>
    </section>
  </form>
</template>

<style scoped>
.endpoint-form,
.form-section,
.model-list,
.model-group {
  display: grid;
  gap: var(--spacing-lg);
}
.endpoint-form {
  padding: var(--spacing-lg);
}
.form-section {
  margin: 0;
}
.form-section h3,
.form-section p {
  margin: 0;
  color: var(--st-text-primary);
  font-size: 15px;
}
.section-header,
.model-row,
.model-group__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}
.section-header__actions,
.model-group__actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
.section-header span,
.model-group__header small,
.empty-text {
  color: var(--st-text-secondary);
}
.model-list {
  gap: var(--spacing-sm);
}
.model-group {
  gap: var(--spacing-sm);
  border: 1px solid var(--st-border);
  padding: var(--spacing-md);
}
.model-group__header {
  min-width: 0;
}
.model-group__header code {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.model-row {
  min-width: 0;
  padding: var(--spacing-sm) 0 0 var(--spacing-md);
  border-top: 1px solid var(--st-border-divider);
}
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
