<template>
  <div class="relative flex h-full w-full items-center justify-center">
    <div class="relative z-1 flex h-full w-full flex-col items-center justify-start gap-4 rounded-xl border-none p-4">
      <div
        class="flex w-full items-center justify-between gap-4 overflow-visible rounded-2xl border border-border-secondary px-4 py-2 shadow-md"
      >
        <div class="flex flex-1 flex-wrap items-center gap-4 p-1">
          <Settings :size="24" class="text-accent" />
          <div>
            <h1 class="m-0 text-xl font-semibold tracking-tight text-main">{{ t('update.title') }}</h1>
            <p class="m-0 text-xs text-secondary">{{ t('update.subtitle') }}</p>
          </div>
        </div>
        <div class="flex flex-wrap gap-3 overflow-visible">
          <CustomButton type="secondary" :icon="Settings" :text="t('navigation.settings')" @click="goToSettings" />
        </div>
      </div>

      <div
        class="relative flex h-full w-full flex-1 items-center justify-center overflow-hidden rounded-2xl border border-border-secondary p-4 shadow-md"
      >
        <UpdateManagerCard :is-standalone="true" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import { useRouter } from 'vue-router'

import CustomButton from '@/components/common/CustomButton.vue'

import UpdateManagerCard from '../components/dashboard/UpdateManagerCard.vue'
import { useTranslation } from '../composables/useI18n'

const { t } = useTranslation()
const router = useRouter()

const goToSettings = () => {
  router.push('/settings')
}
</script>

<style scoped>
.update-view {
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
  min-height: calc(100vh - 4rem);
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 2rem;
  gap: 2rem;
}

.header-content {
  flex: 1;
}

.view-header h1 {
  font-size: 2.5rem;
  font-weight: 700;
  color: var(--color-text);
  margin: 0 0 0.5rem 0;
}

.view-subtitle {
  font-size: 1.125rem;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.6;
}

.header-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.settings-link {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  text-decoration: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.9rem;
}

.settings-link:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.update-content {
  display: grid;
  grid-template-columns: 1fr;
  gap: 2rem;
}

.update-info-section {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
  margin-top: 2rem;
}

.info-card {
  padding: 1.5rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
}

.info-card:hover {
  border-color: var(--color-primary-light);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.info-card h3 {
  margin: 0 0 1rem 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
}

.info-card p {
  margin: 0;
  color: var(--color-text-secondary);
  line-height: 1.6;
}

.info-list {
  margin: 0;
  padding-left: 1.5rem;
  color: var(--color-text-secondary);
}

.info-list li {
  margin-bottom: 0.5rem;
  line-height: 1.5;
}

.info-list li:last-child {
  margin-bottom: 0;
}

/* Dark theme adjustments */
:root.dark .info-card,
:root.auto.dark .info-card {
  background: var(--color-background-secondary);
  border-color: var(--color-border-dark);
}

:root.dark .info-card:hover,
:root.auto.dark .info-card:hover {
  border-color: var(--color-primary);
}

@media (max-width: 768px) {
  .update-view {
    padding: 1rem;
  }

  .view-header {
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
  }

  .view-header h1 {
    font-size: 2rem;
  }

  .view-subtitle {
    font-size: 1rem;
  }

  .header-actions {
    justify-content: flex-start;
  }

  .update-info-section {
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .info-card {
    padding: 1rem;
  }
}

@media (max-width: 480px) {
  .view-header h1 {
    font-size: 1.75rem;
  }
}
</style>
