<script setup lang="ts">
import { onMounted, ref } from 'vue'

import CoreMonitorCard from '../components/dashboard/CoreMonitorCard.vue'
import DocumentationCard from '../components/dashboard/DocumentationCard.vue'
import QuickActionsCard from '../components/dashboard/QuickActionsCard.vue'
import VersionManagerCard from '../components/dashboard/VersionManagerCard.vue'
import { useAppStore } from '../stores/app'

const appStore = useAppStore()

const isLoading = ref(true)

const serviceStatus = ref({
  isRunning: false,
})

onMounted(async () => {
  serviceStatus.value.isRunning = appStore.isCoreRunning
  isLoading.value = false
})
</script>

<template>
  <div class="relative flex h-full w-full items-center justify-center">
    <div class="relative z-1 flex h-full w-full flex-col items-center justify-start gap-4 rounded-xl border-none p-4">
      <div class="flex w-full items-center justify-center border border-border rounded-xl gap-2 shadow-md">
        <QuickActionsCard />
      </div>
      <div class="dashboard-card-wrapper">
        <CoreMonitorCard />
      </div>
      <div class="dashboard-card-wrapper">
        <VersionManagerCard />
      </div>
      <div class="dashboard-card-wrapper">
        <DocumentationCard />
      </div>
    </div>
  </div>
</template>

<style scoped src="./css/HomeView.css"></style>
