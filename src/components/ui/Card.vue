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
  variant: 'default',
  hover: false,
  interactive: false
})
</script>

<style scoped>
.card {
  background: var(--color-surface);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 16px;
  border: 1px solid var(--color-border-secondary);
  box-shadow: var(--shadow-md);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.card--elevated {
  box-shadow: var(--shadow-lg);
}

.card--outlined {
  border: 2px solid var(--color-border);
  box-shadow: var(--shadow-sm);
}

.card--glass {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(40px);
  -webkit-backdrop-filter: blur(40px);
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
  transform: translateY(-4px);
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

@media (prefers-color-scheme: dark) {
  .card--hover:hover,
  .card--interactive:hover {
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 10px 10px -5px rgba(0, 0, 0, 0.3),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
  }
}

.card--interactive {
  cursor: pointer;
}

.card--interactive:active {
  transform: translateY(-2px);
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
