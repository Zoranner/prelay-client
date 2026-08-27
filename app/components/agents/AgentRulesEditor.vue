<script setup lang="ts">
import { MarkdownViewer, Textarea } from "@stellar/ui";

const rules = defineModel<string>({ required: true });
const editorElement = ref<HTMLElement | null>(null);
const previewElement = ref<HTMLElement | null>(null);
let editorTextarea: HTMLTextAreaElement | null = null;
let scrollSyncing = false;

function syncRulesScroll(source: HTMLElement, target: HTMLElement) {
  if (scrollSyncing) return;
  const sourceRange = source.scrollHeight - source.clientHeight;
  const targetRange = target.scrollHeight - target.clientHeight;
  if (sourceRange <= 0 || targetRange <= 0) return;

  scrollSyncing = true;
  target.scrollTop = (source.scrollTop / sourceRange) * targetRange;
  requestAnimationFrame(() => {
    scrollSyncing = false;
  });
}

function onEditorScroll() {
  if (editorTextarea && previewElement.value) {
    syncRulesScroll(editorTextarea, previewElement.value);
  }
}

function onPreviewScroll() {
  if (previewElement.value && editorTextarea) {
    syncRulesScroll(previewElement.value, editorTextarea);
  }
}

function unbindScroll() {
  editorTextarea?.removeEventListener("scroll", onEditorScroll);
  previewElement.value?.removeEventListener("scroll", onPreviewScroll);
  editorTextarea = null;
}

function bindScroll() {
  unbindScroll();
  editorTextarea = editorElement.value?.querySelector("textarea") ?? null;
  editorTextarea?.addEventListener("scroll", onEditorScroll, { passive: true });
  previewElement.value?.addEventListener("scroll", onPreviewScroll, {
    passive: true,
  });
}

onMounted(() => {
  void nextTick().then(bindScroll);
});

onBeforeUnmount(unbindScroll);
</script>

<template>
  <section class="agent-rules">
    <div ref="editorElement" class="agent-rules__editor">
      <Textarea
        v-model="rules"
        class="agent-rules__input"
        aria-label="编辑全局规则"
        :rows="18"
        resize="none"
      />
    </div>
    <div ref="previewElement" class="agent-rules__preview">
      <MarkdownViewer :content="rules" class="agent-rules__markdown" />
    </div>
  </section>
</template>

<style scoped>
.agent-rules {
  display: grid;
  flex: 1;
  width: 100%;
  min-width: 0;
  min-height: 0;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  grid-template-rows: minmax(0, 1fr);
  gap: var(--spacing-xl);
  overflow: hidden;
  padding: 0 0 var(--spacing-lg);
}

.agent-rules__editor,
.agent-rules__preview {
  min-width: 0;
  min-height: 0;
}

.agent-rules__editor {
  display: grid;
  min-height: 0;
  grid-template-rows: minmax(0, 1fr);
}

.agent-rules__preview {
  overflow-y: auto;
}

.agent-rules__input {
  height: 100%;
  min-height: 0;
}

.agent-rules__input :deep(textarea) {
  height: 100%;
}

.agent-rules__markdown {
  min-width: 0;
}
</style>
