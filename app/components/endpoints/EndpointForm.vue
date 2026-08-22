<script setup lang="ts">
import { Button, Input, Popover, Select, useNotification } from "stellar-ui";
import type { EndpointModel, Provider, RelayEndpoint } from "~/stores/relay";

const props = defineProps<{
  endpoint?: RelayEndpoint | null;
  providers: Provider[];
  pending?: boolean;
}>();

const emit = defineEmits<{
  save: [
    payload: {
      id?: string;
      name: string;
      protocol: string;
      models: EndpointModel[];
    },
  ];
  cancel: [];
}>();

type ModelForm = {
  provider_id: string;
  upstream_model: string;
  model_name: string;
};

type ModelGroup = {
  name: string;
  mappings: Array<{ model: EndpointModel; index: number }>;
};

const name = ref("");
const protocol = ref("openai");
const models = ref<EndpointModel[]>([]);
const newModelForm = ref<ModelForm>(emptyModelForm());
const newProviderForm = ref<ModelForm>(emptyModelForm());
const showAddModel = ref(false);
const activeProviderGroup = ref<string | null>(null);
const notifications = useNotification();
const availableProviders = computed(() =>
  props.providers.filter((provider) => provider.models.length > 0),
);
const modelGroups = computed<ModelGroup[]>(() => {
  const groups = new Map<string, ModelGroup>();
  models.value.forEach((model, index) => {
    const name = model.model_name.trim() || model.upstream_model.trim();
    const group = groups.get(name) ?? { name, mappings: [] };
    group.mappings.push({ model, index });
    groups.set(name, group);
  });
  return [...groups.values()];
});
const providerOptions = computed(() => [
  { label: "选择供应商", value: "" },
  ...availableProviders.value.map((provider) => ({
    label: provider.name,
    value: provider.id,
  })),
]);

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
  },
  { immediate: true },
);

function emptyModelForm(): ModelForm {
  return { provider_id: "", upstream_model: "", model_name: "" };
}

function modelsForProvider(providerId: string) {
  return (
    availableProviders.value.find((provider) => provider.id === providerId)
      ?.models ?? []
  );
}

function availableUpstreamModels(providerId: string, group?: ModelGroup) {
  const usedModelNames = new Set(
    group?.mappings
      .filter((mapping) => mapping.model.provider_id === providerId)
      .map((mapping) => mapping.model.upstream_model) ?? [],
  );
  return modelsForProvider(providerId).filter(
    (model) => !usedModelNames.has(model.model_name),
  );
}

function providerForModel(model: Pick<EndpointModel, "provider_id">) {
  return props.providers.find((provider) => provider.id === model.provider_id);
}

function upstreamModelOptions(providerId: string, group?: ModelGroup) {
  const models = availableUpstreamModels(providerId, group);
  return [
    {
      label: providerId
        ? models.length
          ? "选择上游模型"
          : "无可用模型"
        : "先选择供应商",
      value: "",
    },
    ...models.map((model) => ({
      label: model.model_name,
      value: model.model_name,
    })),
  ];
}

function selectProvider(form: ModelForm, group?: ModelGroup) {
  form.upstream_model =
    availableUpstreamModels(form.provider_id, group)[0]?.model_name ?? "";
}

function addMapping(form: ModelForm, fixedModelName?: string) {
  const upstreamModel = form.upstream_model.trim();
  const modelName =
    fixedModelName ?? (form.model_name.trim() || upstreamModel);
  if (!form.provider_id || !upstreamModel) {
    notifications.danger("请选择供应商和上游模型。", {
      title: "模型配置不完整",
    });
    return false;
  }
  if (!modelsForProvider(form.provider_id).some((item) => item.model_name === upstreamModel)) {
    notifications.danger("请选择该供应商已配置的模型。", {
      title: "上游模型无效",
    });
    return false;
  }
  models.value.push({
    provider_id: form.provider_id,
    upstream_model: upstreamModel,
    model_name: modelName,
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

function submit() {
  if (!name.value.trim()) {
    notifications.danger("请填写接入点名称。", { title: "接入点配置不完整" });
    return;
  }
  if (!models.value.length) {
    notifications.danger("请至少新增一个模型。", { title: "接入点配置不完整" });
    return;
  }
  emit("save", {
    ...(props.endpoint ? { id: props.endpoint.id } : {}),
    name: name.value.trim(),
    protocol: protocol.value,
    models: models.value.map(({ provider_id, upstream_model, model_name }) => ({
      provider_id,
      upstream_model,
      model_name,
    })),
  });
}
</script>

<template>
  <form id="endpoint-form" class="endpoint-form" @submit.prevent="submit">
    <section class="form-section">
      <h3>接入点配置</h3>
      <Input v-model="name" label="名称" placeholder="Codex 主接入点" />
    </section>
    <section class="form-section">
      <div class="section-header">
        <h3>模型列表</h3>
        <div class="section-header__actions">
          <span>{{ modelGroups.length }} 个</span>
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
                <Select
                  v-model="newModelForm.provider_id"
                  label="供应商"
                  :options="providerOptions"
                  @change="selectProvider(newModelForm)"
                />
                <Select
                  v-model="newModelForm.upstream_model"
                  label="上游模型"
                  :disabled="!modelsForProvider(newModelForm.provider_id).length"
                  :options="upstreamModelOptions(newModelForm.provider_id)"
                />
                <Input
                  v-model="newModelForm.model_name"
                  label="对外模型名"
                  placeholder="不填则使用上游模型名"
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
        <div
          v-for="group in modelGroups"
          :key="group.name"
          class="model-group"
        >
          <div class="model-group__header">
            <code>{{ group.name }}</code>
            <div class="model-group__actions">
              <small>{{ group.mappings.length }} 个供应商</small>
              <Popover
                :model-value="activeProviderGroup === group.name"
                position="bottom"
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
                      :disabled="!availableUpstreamModels(newProviderForm.provider_id, group).length"
                      :options="upstreamModelOptions(newProviderForm.provider_id, group)"
                    />
                  </div>
                </template>
                <template #footer>
                  <Button
                    variant="primary"
                    type="button"
                    :disabled="pending"
                    @click="addProvider(group.name)"
                    >确认</Button
                  >
                </template>
              </Popover>
            </div>
          </div>
          <div
            v-for="mapping in group.mappings"
            :key="
              mapping.model.id ??
              `${mapping.model.provider_id}-${mapping.model.upstream_model}-${mapping.index}`
            "
            class="model-row"
          >
            <small
              >{{ providerForModel(mapping.model)?.name ?? "已删除供应商" }} /
              {{ mapping.model.upstream_model }}</small
            >
            <Button
              square
              size="small"
              variant="danger"
              icon="ph:trash"
              aria-label="删除供应商映射"
              title="删除供应商映射"
              @click="removeModel(mapping.index)"
            />
          </div>
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
.form-section {
  margin: 0;
  border: 1px solid var(--st-border);
  padding: var(--spacing-lg);
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
.model-row small,
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
.model-row small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.model-popover {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-md);
}
.model-popover > :nth-child(3) {
  grid-column: span 2;
}
@media (max-width: 640px) {
  .model-popover {
    grid-template-columns: 1fr;
  }
  .model-popover > :nth-child(3) {
    grid-column: auto;
  }
}
</style>
