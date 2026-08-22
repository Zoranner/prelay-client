<script setup lang="ts">
const desktopPreferencesDialog = useDesktopPreferencesDialog();

onMounted(() => {
  desktopPreferencesDialog.open();
  void navigateTo("/");
});
</script>

<template><div /></template>
