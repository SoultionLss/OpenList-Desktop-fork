<template>
  <div v-if="isOpen" class="dialog-overlay" @click="onCancel">
    <div class="dialog-container" @click.stop>
      <div class="dialog-header">
        <h3 class="dialog-title">{{ title }}</h3>
      </div>
      <div class="dialog-content">
        <p class="dialog-message">{{ message }}</p>
      </div>
      <div class="dialog-actions">
        <button class="dialog-btn cancel-btn" @click="onCancel">
          {{ cancelText }}
        </button>
        <button class="dialog-btn confirm-btn" :class="confirmButtonClass" @click="onConfirm">
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  isOpen: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'default' | 'danger'
}

interface Emits {
  (e: 'confirm'): void
  (e: 'cancel'): void
}

const props = withDefaults(defineProps<Props>(), {
  confirmText: 'Confirm',
  cancelText: 'Cancel',
  variant: 'default'
})

const emit = defineEmits<Emits>()

const confirmButtonClass = computed(() => {
  return props.variant === 'danger' ? 'danger' : 'primary'
})

const onConfirm = () => {
  emit('confirm')
}

const onCancel = () => {
  emit('cancel')
}
</script>

<script lang="ts">
import { computed } from 'vue'
export default {
  name: 'ConfirmDialog'
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-container {
  background: white;
  border-radius: 0.75rem;
  box-shadow:
    0 20px 25px -5px rgba(0, 0, 0, 0.1),
    0 10px 10px -5px rgba(0, 0, 0, 0.04);
  max-width: 28rem;
  width: 90%;
  max-height: 80vh;
  overflow: hidden;
}

:root.dark .dialog-container,
:root.auto.dark .dialog-container {
  background: rgb(31 41 55);
  border: 1px solid rgb(55 65 81);
}

.dialog-header {
  padding: 1.5rem 1.5rem 0 1.5rem;
}

.dialog-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: rgb(17 24 39);
  margin: 0;
}

:root.dark .dialog-title,
:root.auto.dark .dialog-title {
  color: rgb(243 244 246);
}

.dialog-content {
  padding: 1rem 1.5rem;
}

.dialog-message {
  color: rgb(107 114 128);
  line-height: 1.6;
  margin: 0;
}

:root.dark .dialog-message,
:root.auto.dark .dialog-message {
  color: rgb(156 163 175);
}

.dialog-actions {
  display: flex;
  gap: 0.75rem;
  padding: 0 1.5rem 1.5rem 1.5rem;
  justify-content: flex-end;
}

.dialog-btn {
  padding: 0.5rem 1rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  font-weight: 500;
  border: none;
  cursor: pointer;
  min-width: 4rem;
}

.cancel-btn {
  background: rgb(243 244 246);
  color: rgb(75 85 99);
  border: 1px solid rgb(209 213 219);
}

.cancel-btn:hover {
  background: rgb(229 231 235);
}

:root.dark .cancel-btn,
:root.auto.dark .cancel-btn {
  background: rgb(55 65 81);
  color: rgb(209 213 219);
  border-color: rgb(75 85 99);
}

:root.dark .cancel-btn:hover,
:root.auto.dark .cancel-btn:hover {
  background: rgb(75 85 99);
}

.confirm-btn.primary {
  background: rgb(59 130 246);
  color: white;
}

.confirm-btn.primary:hover {
  background: rgb(37 99 235);
}

.confirm-btn.danger {
  background: rgb(239 68 68);
  color: white;
}

.confirm-btn.danger:hover {
  background: rgb(220 38 38);
}
</style>
