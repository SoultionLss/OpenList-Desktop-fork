<script setup lang="ts">
import { computed, ref, onMounted, nextTick, watch } from 'vue'
import { useAppStore } from '../../stores/app'
import { useTranslation } from '../../composables/useI18n'
import { ChevronLeft, ChevronRight, X, Check, Play, FileText, Settings, HardDrive, Home } from 'lucide-vue-next'

const store = useAppStore()
const { t } = useTranslation()

const tutorialSteps = computed(() => [
  {
    title: t('tutorial.welcome.title'),
    content: t('tutorial.welcome.content'),
    target: '.app-title',
    position: 'center',
    showNext: true,
    showSkip: true,
    icon: Home
  },
  {
    title: t('tutorial.navigation.title'),
    content: t('tutorial.navigation.content'),
    target: '.nav-menu',
    position: 'right',
    showNext: true,
    showPrev: true,
    showSkip: true,
    icon: HardDrive
  },
  {
    title: t('tutorial.service.title'),
    content: t('tutorial.service.content'),
    target: '.service-management-card',
    position: 'top',
    showNext: true,
    showPrev: true,
    showSkip: true,
    icon: Play
  },
  {
    title: t('tutorial.openlist.title'),
    content: t('tutorial.openlist.content'),
    target: '.quick-actions-card',
    position: 'top',
    showNext: true,
    showPrev: true,
    showSkip: true,
    icon: HardDrive
  },
  {
    title: t('tutorial.documentation.title'),
    content: t('tutorial.documentation.content'),
    target: '.documentation-card',
    position: 'bottom',
    showNext: true,
    showPrev: true,
    showSkip: true,
    icon: FileText
  },
  {
    title: t('tutorial.settings.title'),
    content: t('tutorial.settings.content'),
    target: '.nav-item[href="/settings"]',
    position: 'right',
    showPrev: true,
    showComplete: true,
    icon: Settings
  }
])

const currentStep = computed(() => tutorialSteps.value[store.tutorialStep] || tutorialSteps.value[0])

const highlightStyle = ref({})

const updateHighlight = async () => {
  await nextTick()

  if (!currentStep.value.target) {
    highlightStyle.value = {}
    return
  }

  const targetElement = document.querySelector(currentStep.value.target) as HTMLElement
  if (!targetElement) {
    highlightStyle.value = {}
    return
  }

  const rect = targetElement.getBoundingClientRect()
  const padding = 8

  highlightStyle.value = {
    top: `${rect.top - padding}px`,
    left: `${rect.left - padding}px`,
    width: `${rect.width + padding * 2}px`,
    height: `${rect.height + padding * 2}px`
  }
}

const getTooltipStyle = () => {
  if (!currentStep.value.target)
    return {
      top: '50%',
      left: '50%',
      transform: 'translate(-50%, -50%)'
    }

  const targetElement = document.querySelector(currentStep.value.target) as HTMLElement
  if (!targetElement)
    return {
      top: '50%',
      left: '50%',
      transform: 'translate(-50%, -50%)'
    }

  const rect = targetElement.getBoundingClientRect()
  const position = currentStep.value.position || 'bottom'
  const offset = 16
  const tooltipWidth = 320
  const tooltipHeight = 200

  let style: any = {}

  let adjustedPosition = position
  if (position === 'left' && rect.left < tooltipWidth + offset) {
    adjustedPosition = 'right'
  } else if (position === 'right' && rect.right + tooltipWidth + offset > window.innerWidth) {
    adjustedPosition = 'left'
  } else if (position === 'top' && rect.top < tooltipHeight + offset) {
    adjustedPosition = 'bottom'
  } else if (position === 'bottom' && rect.bottom + tooltipHeight + offset > window.innerHeight) {
    adjustedPosition = 'top'
  }

  switch (adjustedPosition) {
    case 'center':
      style = {
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)'
      }
      break
    case 'top':
      style = {
        bottom: `${window.innerHeight - rect.top + offset}px`,
        left: `${rect.left + rect.width / 2}px`,
        transform: 'translateX(-50%)'
      }
      break
    case 'bottom':
      style = {
        top: `${rect.bottom + offset}px`,
        left: `${rect.left + rect.width / 2}px`,
        transform: 'translateX(-50%)'
      }
      break
    case 'left':
      style = {
        top: `${rect.top + rect.height / 2}px`,
        right: `${window.innerWidth - rect.left + offset}px`,
        transform: 'translateY(-50%)'
      }
      break
    case 'right':
      style = {
        top: `${rect.top + rect.height / 2}px`,
        left: `${rect.right + offset}px`,
        transform: 'translateY(-50%)'
      }
      break
    case 'bottom-right':
      style = {
        top: `${rect.bottom + offset}px`,
        left: `${Math.max(16, rect.right - tooltipWidth)}px`
      }
      break
    default:
      style = {
        top: `${rect.bottom + offset}px`,
        left: `${rect.left + rect.width / 2}px`,
        transform: 'translateX(-50%)'
      }
  }
  if (style.left && !style.transform?.includes('translateX')) {
    const leftPos = parseInt(style.left)
    if (leftPos + tooltipWidth > window.innerWidth) {
      style.left = `${window.innerWidth - tooltipWidth - 16}px`
    }
    if (leftPos < 16) {
      style.left = '16px'
    }
  }

  if (style.top && !style.transform?.includes('translateY')) {
    const topPos = parseInt(style.top)
    if (topPos + tooltipHeight > window.innerHeight) {
      style.top = `${window.innerHeight - tooltipHeight - 16}px`
    }
    if (topPos < 16) {
      style.top = '16px'
    }
  }

  if (style.bottom) {
    const bottomPos = parseInt(style.bottom)
    if (window.innerHeight - bottomPos - tooltipHeight < 16) {
      delete style.bottom
      style.top = '16px'
    }
  }

  if (style.right) {
    const rightPos = parseInt(style.right)
    if (window.innerWidth - rightPos - tooltipWidth < 16) {
      delete style.right
      style.left = '16px'
      delete style.transform
    }
  }

  if (style.transform?.includes('translate(-50%, -50%)')) {
    return style
  }

  if (style.transform?.includes('translateX(-50%)') && style.left) {
    const leftPos = parseInt(style.left)
    const halfWidth = tooltipWidth / 2
    if (leftPos - halfWidth < 16) {
      style.left = `${halfWidth + 16}px`
    }
    if (leftPos + halfWidth > window.innerWidth - 16) {
      style.left = `${window.innerWidth - halfWidth - 16}px`
    }
  }

  if (style.transform?.includes('translateY(-50%)') && style.top) {
    const topPos = parseInt(style.top)
    const halfHeight = tooltipHeight / 2
    if (topPos - halfHeight < 16) {
      style.top = `${halfHeight + 16}px`
    }
    if (topPos + halfHeight > window.innerHeight - 16) {
      style.top = `${window.innerHeight - halfHeight - 16}px`
    }
  }

  return style
}

const handleNext = () => {
  if (store.tutorialStep < tutorialSteps.value.length - 1) {
    store.nextTutorialStep()
    updateHighlight()
  }
}

const handlePrev = () => {
  if (store.tutorialStep > 0) {
    store.prevTutorialStep()
    updateHighlight()
  }
}

const handleSkip = () => {
  store.skipTutorial()
}

const handleComplete = () => {
  store.completeTutorial()
}

const handleClose = () => {
  store.closeTutorial()
}

onMounted(() => {
  updateHighlight()

  watch(
    () => store.tutorialStep,
    () => {
      setTimeout(() => {
        updateHighlight()
      }, 100)
    }
  )

  const handleResize = () => {
    updateHighlight()
  }

  window.addEventListener('resize', handleResize)

  return () => {
    window.removeEventListener('resize', handleResize)
  }
})
</script>

<template>
  <Teleport to="body">
    <div v-if="store.showTutorial" class="tutorial-overlay">
      <div class="tutorial-backdrop" @click="handleClose" />
      <div
        v-if="currentStep.target && currentStep.position !== 'center'"
        class="tutorial-highlight"
        :style="highlightStyle"
      />

      <div class="tutorial-tooltip" :style="getTooltipStyle()">
        <div class="tooltip-header">
          <div class="tooltip-icon">
            <component :is="currentStep.icon" :size="20" />
          </div>
          <h3 class="tooltip-title">{{ currentStep.title }}</h3>
          <button class="tooltip-close" @click="handleClose" :title="t('common.close')">
            <X :size="18" />
          </button>
        </div>

        <div class="tooltip-content">
          <p>{{ currentStep.content }}</p>
        </div>

        <div class="tooltip-footer">
          <div class="step-indicator">
            <span class="step-current">{{ store.tutorialStep + 1 }}</span>
            <span class="step-divider">/</span>
            <span class="step-total">{{ tutorialSteps.length }}</span>
          </div>

          <div class="tutorial-actions">
            <button v-if="currentStep.showSkip" class="btn-skip" @click="handleSkip">
              {{ t('tutorial.skip') }}
            </button>

            <button v-if="currentStep.showPrev" class="btn-prev" @click="handlePrev">
              <ChevronLeft :size="16" />
              {{ t('tutorial.previous') }}
            </button>

            <button v-if="currentStep.showNext" class="btn-next" @click="handleNext">
              {{ t('tutorial.next') }}
              <ChevronRight :size="16" />
            </button>

            <button v-if="currentStep.showComplete" class="btn-complete" @click="handleComplete">
              <Check :size="16" />
              {{ t('tutorial.complete') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.tutorial-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  pointer-events: none;
}

.tutorial-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(2px);
  pointer-events: all;
}

.tutorial-highlight {
  position: absolute;
  border: 2px solid var(--color-accent);
  border-radius: var(--radius-md);
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.2), var(--shadow-lg);
  background: rgba(255, 255, 255, 0.05);
  pointer-events: none;
  transition: all var(--transition-medium);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%,
  100% {
    box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.2), var(--shadow-lg);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(0, 122, 255, 0.1), var(--shadow-xl);
  }
}

.tutorial-tooltip {
  position: absolute;
  width: 320px;
  max-width: calc(100vw - 32px);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  backdrop-filter: blur(20px);
  pointer-events: all;
  animation: tooltipEnter 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  z-index: 10001;
}

@keyframes tooltipEnter {
  0% {
    opacity: 0;
    transform: translateY(16px) scale(0.9);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.tooltip-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px 12px 20px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.tooltip-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--color-accent);
  color: white;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.tooltip-title {
  flex: 1;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.tooltip-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  color: var(--color-text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tooltip-close:hover {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.tooltip-content {
  padding: 16px 20px;
}

.tooltip-content p {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--color-text-secondary);
}

.tooltip-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px 16px 20px;
  border-top: 1px solid var(--color-border-secondary);
}

.step-indicator {
  display: flex;
  align-items: center;
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.step-current {
  font-weight: 600;
  color: var(--color-accent);
}

.step-divider {
  margin: 0 4px;
}

.tutorial-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-skip {
  padding: 6px 12px;
  font-size: 0.75rem;
  background: transparent;
  border: 1px solid var(--color-border);
  color: var(--color-text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-skip:hover {
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
}

.btn-prev,
.btn-next,
.btn-complete {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  font-size: 0.75rem;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-weight: 500;
}

.btn-prev {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.btn-prev:hover {
  background: var(--color-background-tertiary);
}

.btn-next,
.btn-complete {
  background: var(--color-accent);
  color: white;
}

.btn-next:hover,
.btn-complete:hover {
  background: var(--color-accent-hover);
}

.btn-complete {
  background: var(--color-success);
}

.btn-complete:hover {
  background: #2fb344;
}

:root.dark .tutorial-backdrop {
  background: rgba(0, 0, 0, 0.8);
}

:root.dark .tutorial-highlight {
  border-color: var(--color-accent);
  box-shadow: 0 0 0 4px rgba(10, 132, 255, 0.3), var(--shadow-lg);
}

:root.dark .tutorial-tooltip {
  background: var(--color-surface-elevated);
  border-color: var(--color-border);
}

@media (max-width: 768px) {
  .tutorial-tooltip {
    width: 280px;
    max-width: calc(100vw - 24px);
    position: fixed !important;
    top: auto !important;
    bottom: 20px !important;
    left: 50% !important;
    right: auto !important;
    transform: translateX(-50%) !important;
  }

  .tooltip-header {
    padding: 12px 16px 8px 16px;
  }

  .tooltip-content {
    padding: 12px 16px;
  }

  .tooltip-footer {
    padding: 8px 16px 12px 16px;
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
  }

  .tutorial-actions {
    justify-content: space-between;
    width: 100%;
  }

  .step-indicator {
    align-self: center;
  }
}

@media (max-width: 480px) {
  .tutorial-tooltip {
    width: calc(100vw - 32px);
  }

  .tutorial-actions {
    flex-direction: column;
    gap: 8px;
  }

  .btn-prev,
  .btn-next,
  .btn-complete,
  .btn-skip {
    width: 100%;
    justify-content: center;
  }
}
</style>
