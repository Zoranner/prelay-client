<script setup lang="ts">
import HookReviewList from "~/components/extensions/HookReviewList.vue";
import type { HookInstallPreview as HookPreview } from "~/utils/extensionHookPreview";

defineProps<{ preview: HookPreview }>();
</script>

<template>
  <HookReviewList :review="preview.review" :files="preview.files" />
</template>
