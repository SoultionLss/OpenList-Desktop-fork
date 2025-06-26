<template>
  <div class="status-indicator" :class="{ active: isActive }">
    <div class="status-dot"></div>
    <span class="status-text">
      {{ isActive ? activeText : inactiveText }}
    </span>
  </div>
</template>

<script setup lang="ts">
interface Props {
  isActive: boolean
  activeText: string
  inactiveText: string
}

defineProps<Props>()
</script>

<style scoped>
.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.75rem;
  background-color: rgb(243 244 246);
  border-radius: 0.5rem;
  transition-property: color, background-color, border-color, text-decoration-color, fill, stroke;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 150ms;
}

:root.dark .status-indicator,
:root.auto.dark .status-indicator {
  background-color: rgb(31 41 55);
}

.status-indicator.active {
  background-color: rgb(240 253 244);
}

:root.dark .status-indicator.active,
:root.auto.dark .status-indicator.active {
  background-color: rgb(20 83 45 / 0.2);
}

.status-dot {
  width: 0.5rem;
  height: 0.5rem;
  background-color: rgb(156 163 175);
  border-radius: 9999px;
  transition-property: color, background-color, border-color, text-decoration-color, fill, stroke;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 150ms;
}

.status-indicator.active .status-dot {
  background-color: rgb(34 197 94);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.status-text {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

:root.dark .status-text,
:root.auto.dark .status-text {
  color: var(--color-text-secondary);
}

.status-indicator.active .status-text {
  color: rgb(21 128 61);
}

:root.dark .status-indicator.active .status-text,
:root.auto.dark .status-indicator.active .status-text {
  color: rgb(134 239 172);
}
</style>
