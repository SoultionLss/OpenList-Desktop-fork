<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useAppStore } from '../stores/app'

import QuickActionsCard from '../components/dashboard/QuickActionsCard.vue'
import CoreMonitorCard from '../components/dashboard/CoreMonitorCard.vue'
import VersionManagerCard from '../components/dashboard/VersionManagerCard.vue'
import DocumentationCard from '../components/dashboard/DocumentationCard.vue'
import ServiceManagementCard from '../components/dashboard/ServiceManagementCard.vue'

const store = useAppStore()

const isLoading = ref(true)

const serviceStatus = ref({
  isRunning: false
})

const layoutClass = computed(() => ({
  'dashboard-loading': isLoading.value,
  'dashboard-ready': !isLoading.value,
  'service-running': serviceStatus.value.isRunning
}))

onMounted(async () => {
  serviceStatus.value.isRunning = store.isCoreRunning
  isLoading.value = false
})
</script>

<template>
  <div class="dashboard-container" :class="layoutClass">
    <div class="dashboard-grid" :class="{ 'three-column': !isLoading }">
      <div class="dashboard-card-wrapper">
        <QuickActionsCard />
      </div>
      <div class="dashboard-card-wrapper">
        <CoreMonitorCard />
      </div>
      <div class="dashboard-card-wrapper">
        <ServiceManagementCard />
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
