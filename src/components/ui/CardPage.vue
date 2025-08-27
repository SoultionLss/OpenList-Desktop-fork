<template>
  <div class="card" :class="[`card--${variant}`, { 'card--hover': hover, 'card--interactive': interactive }]">
    <div v-if="$slots.header || title" class="card__header">
      <slot name="header">
        <h3 v-if="title" class="card__title">{{ title }}</h3>
      </slot>
      <div v-if="$slots.headerActions" class="card__header-actions">
        <slot name="headerActions" />
      </div>
    </div>

    <div class="card__content">
      <slot />
    </div>

    <div v-if="$slots.footer" class="card__footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string
  variant?: 'default' | 'elevated' | 'outlined' | 'glass'
  hover?: boolean
  interactive?: boolean
}

withDefaults(defineProps<Props>(), {
  title: '',
  variant: 'default',
  hover: false,
  interactive: false
})
</script>

<style scoped>
.card {
  background: var(--color-surface);
  border-radius: 16px;
  border: 1px solid var(--color-border-secondary);
  position: relative;
  overflow: hidden;
}

.card--elevated {
  box-shadow: var(--shadow-sm);
}

.card--outlined {
  border: 2px solid var(--color-border);
}

.card--glass {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

@media (prefers-color-scheme: dark) {
  .card--glass {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.05);
  }
}

/* Interactive states */
.card--hover:hover,
.card--interactive:hover {
  background: var(--color-surface-elevated);
  border-color: rgba(59, 130, 246, 0.2);
}

@media (prefers-color-scheme: dark) {
  .card--hover:hover,
  .card--interactive:hover {
    background: var(--color-surface-elevated);
    border-color: rgba(59, 130, 246, 0.2);
  }
}

.card--interactive {
  cursor: pointer;
}

.card--interactive:active {
  opacity: 0.9;
}

/* Card structure */
.card__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem 0 1rem;
  margin-bottom: 0.5rem;
}

.card__title {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
  letter-spacing: -0.025em;
}

.card__header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.card__content {
  padding: 0 1rem 1rem 1rem;
}

.card__footer {
  padding: 0 1rem 0.75rem 1rem;
  border-top: 1px solid var(--color-border-secondary);
  margin-top: 0.5rem;
  padding-top: 0.5rem;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .card {
    border-radius: 12px;
  }

  .card__header,
  .card__content,
  .card__footer {
    padding-left: 0.75rem;
    padding-right: 0.75rem;
  }

  .card__header {
    padding-top: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .card__content {
    padding-bottom: 0.75rem;
  }

  .card__footer {
    padding-bottom: 0.75rem;
    padding-top: 0.5rem;
    margin-top: 0.5rem;
  }

  .card__title {
    font-size: 1rem;
  }
}
</style>
