<template>
  <div class="window-controls">
    <button class="control-btn minimize" :title="t('common.minimize')" @click="$emit('minimize')">
      <Minimize2 :size="12" />
    </button>
    <button class="control-btn maximize" :title="t('common.maximize')" @click="$emit('maximize')">
      <Maximize2 :size="12" />
    </button>
    <button class="control-btn close" :title="t('common.close')" @click="$emit('close')">
      <X :size="12" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { Maximize2, Minimize2, X } from 'lucide-vue-next'

import { useTranslation } from '../../composables/useI18n'

const { t } = useTranslation()

defineEmits<{
  minimize: []
  maximize: []
  close: []
}>()
</script>

<style scoped>
.window-controls {
  display: flex;
  gap: 0.25rem;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: none;
  background: transparent;
  color: rgb(107 114 128);
  border-radius: 0.25rem;
  cursor: pointer;
}

.control-btn:hover {
  background: rgb(243 244 246);
  color: rgb(75 85 99);
}

@media (prefers-color-scheme: dark) {
  .control-btn {
    color: rgb(156 163 175);
  }

  .control-btn:hover {
    background: rgb(55 65 81);
    color: rgb(209 213 219);
  }
}

.control-btn.close:hover {
  background: rgb(239 68 68);
  color: white;
}
</style>
