<template>
  <Transition name="notification" appear>
    <div v-if="visible" class="update-notification" :class="type">
      <div class="notification-content">
        <div class="notification-icon">
          <component :is="getIcon()" :size="20" />
        </div>
        <div class="notification-text">
          <div class="notification-title">{{ title }}</div>
          <div v-if="message" class="notification-message">{{ message }}</div>
        </div>
        <div class="notification-actions">
          <button v-if="showAction" class="action-btn" @click="$emit('action')">
            {{ actionText }}
          </button>
          <button class="dismiss-btn" @click="$emit('dismiss')">
            <X :size="16" />
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { AlertCircle, CheckCircle, Download, X } from 'lucide-vue-next'

interface Props {
  visible: boolean
  type?: 'info' | 'success' | 'warning' | 'error'
  title: string
  message: string
  showAction?: boolean
  actionText?: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  showAction: false,
  actionText: 'Action'
})

defineEmits<{
  action: []
  dismiss: []
}>()

const getIcon = () => {
  switch (props.type) {
    case 'success':
      return CheckCircle
    case 'warning':
    case 'error':
      return AlertCircle
    case 'info':
    default:
      return Download
  }
}
</script>

<style scoped>
.update-notification {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9999;
  min-width: 300px;
  max-width: 400px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-left: 4px solid var(--color-primary);
}

.update-notification.info {
  border-left-color: var(--color-info);
}

.update-notification.success {
  border-left-color: var(--color-success);
}

.update-notification.warning {
  border-left-color: var(--color-warning);
}

.update-notification.error {
  border-left-color: var(--color-error);
}

.notification-content {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 1rem;
}

.notification-icon {
  flex-shrink: 0;
  margin-top: 0.125rem;
}

.notification-text {
  flex: 1;
  min-width: 0;
}

.notification-title {
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 0.25rem;
}

.notification-message {
  font-size: 0.9rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.notification-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.action-btn {
  background: var(--color-primary);
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
}

.action-btn:hover {
  background: var(--color-primary-dark);
}

.dismiss-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 4px;
  color: var(--color-text-secondary);
}

.dismiss-btn:hover {
  color: var(--color-text);
  background: var(--color-surface);
}

.notification-enter-from {
  opacity: 0;
}

.notification-leave-to {
  opacity: 0;
}
</style>
