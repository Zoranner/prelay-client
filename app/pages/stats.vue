<script setup lang="ts">
import type { Activity } from "~/stores/relay";
import { Button } from "@stellar/ui";
import RequestTable from "~/components/activity/RequestTable.vue";
import PanelSection from "~/components/shell/PanelSection.vue";

const { pending, invokeCommand } = useRelayCommand();
const activities = ref<Activity[]>([]);
const limit = ref(100);
async function loadActivities() {
  try {
    activities.value = await invokeCommand<Activity[]>("stats_activities", {
      limit: limit.value,
    });
  } catch {
    // The command composable exposes the stable error to this view.
  }
}

onMounted(loadActivities);
</script>

<template>
  <main class="page-dashboard">
    <PanelSection title="活动">
      <template #header-actions>
        <Button
          variant="primary"
          icon="ph:arrows-clockwise"
          :disabled="pending"
          @click="loadActivities"
        >
          {{ pending ? "刷新中..." : "刷新" }}
        </Button>
      </template>
      <RequestTable
        v-model:limit="limit"
        :pending="pending"
        :requests="activities"
        @reload="loadActivities"
      />
    </PanelSection>
  </main>
</template>

<style scoped>
.page-dashboard {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  padding: var(--pr-dashboard-padding);
}
</style>
