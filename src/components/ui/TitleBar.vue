<template>
  <div class="title-bar" data-tauri-drag-region @dblclick="handleDoubleClick">
    <div data-tauri-drag-region class="title-bar-content">
      <div class="title-left">
        <span class="app-title">{{ appTitle }}</span>
      </div>

      <div class="title-center">
        <slot name="center" />
      </div>

      <div class="title-right" @dblclick.stop>
        <WindowControls @minimize="handleMinimize" @maximize="handleMaximize" @close="handleClose" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ref, onMounted, onUnmounted } from 'vue'
import WindowControls from './WindowControls.vue'
interface Props {
  appTitle?: string
}

withDefaults(defineProps<Props>(), {
  appTitle: 'OpenList Desktop'
})

const isMaximized = ref(false)
let appWindow: any = null
let unlistenResize: (() => void) | null = null

onMounted(async () => {
  appWindow = getCurrentWindow()

  try {
    unlistenResize = await appWindow.listen('tauri://resize', async () => {
      isMaximized.value = await appWindow.isMaximized()
    })

    isMaximized.value = await appWindow.isMaximized()
  } catch (error) {
    console.error('Error setting up window listeners:', error)
  }
})

onUnmounted(() => {
  if (unlistenResize) {
    unlistenResize()
  }
})

const handleMinimize = async () => {
  try {
    const appWindow = getCurrentWindow()
    await appWindow.minimize()
  } catch (error) {
    console.error('Error minimizing window:', error)
  }
}

const handleMaximize = async () => {
  try {
    const appWindow = getCurrentWindow()
    const currentMaximized = await appWindow.isMaximized()
    if (currentMaximized) {
      await appWindow.unmaximize()
    } else {
      await appWindow.maximize()
    }
    isMaximized.value = await appWindow.isMaximized()
  } catch (error) {
    console.error('Error toggling window maximize:', error)
  }
}

const handleClose = async () => {
  try {
    const appWindow = getCurrentWindow()
    await appWindow.close()
  } catch (error) {
    console.error('Error closing window:', error)
  }
}

const handleDoubleClick = async () => {
  await handleMaximize()
}
</script>

<style scoped>
.title-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 32px;
  background: var(--color-background-secondary);
  border-bottom: 1px solid var(--color-border);
  z-index: 1000;
  user-select: none;
  -webkit-user-select: none;
}

.title-bar-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 8px;
}

.title-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 0 0 auto;
}

.app-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  color: var(--color-text-primary);
}

.app-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  opacity: 0.8;
}

.title-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 16px;
}

.title-right {
  flex: 0 0 auto;
}

@media (platform: macos) {
  .title-bar {
    padding-left: 78px;
    background: var(--color-background-secondary);
  }
}

.title-right {
  pointer-events: auto;
}

.title-right * {
  pointer-events: auto;
}

.title-bar * {
  user-select: none;
  -webkit-user-select: none;
}
</style>
