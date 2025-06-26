import { onMounted, onUnmounted } from 'vue'

import { TauriAPI } from '../api/tauri'
import { useAppStore } from '../stores/app'
import { useServiceActions } from './useServiceActions'

export const useTray = () => {
  const { startService, stopService, restartService } = useServiceActions()
  const store = useAppStore()

  let unlistenTrayActions: (() => void) | null = null
  const updateTrayMenu = async (serviceRunning: boolean) => {
    try {
      await TauriAPI.updateTrayMenuDelayed(serviceRunning)
    } catch (error) {
      console.error('Failed to update tray menu:', error)
    }
  }
  const handleTrayServiceAction = async (action: string) => {
    console.log('Tray service action:', action)

    try {
      switch (action) {
        case 'start':
          await startService()
          setTimeout(async () => {
            await updateTrayMenu(store.serviceStatus.running)
          }, 5000)
          break
        case 'stop':
          await stopService()
          setTimeout(async () => {
            await updateTrayMenu(store.serviceStatus.running)
          }, 5000)
          break
        case 'restart':
          await restartService()
          setTimeout(async () => {
            await updateTrayMenu(store.serviceStatus.running)
          }, 5000)
          break
        default:
          console.warn('Unknown tray service action:', action)
      }
    } catch (error) {
      console.error(`Failed to execute tray action '${action}':`, error)
      setTimeout(async () => {
        await updateTrayMenu(store.serviceStatus.running)
      }, 3000)
    }
  }
  const initTrayListeners = async () => {
    try {
      unlistenTrayActions = await TauriAPI.listenToTrayServiceActions(handleTrayServiceAction)

      await TauriAPI.forceUpdateTrayMenu(store.serviceStatus.running)
      console.log('Tray listeners initialized and menu updated')
    } catch (error) {
      console.error('Failed to initialize tray listeners:', error)
    }
  }

  const cleanupTrayListeners = () => {
    if (unlistenTrayActions) {
      unlistenTrayActions()
      unlistenTrayActions = null
    }
  }

  onMounted(() => {
    initTrayListeners()
  })

  onUnmounted(() => {
    cleanupTrayListeners()
  })

  return {
    updateTrayMenu,
    initTrayListeners,
    cleanupTrayListeners
  }
}
