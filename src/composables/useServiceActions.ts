import { useAppStore } from '../stores/app'

export const useServiceActions = () => {
  const store = useAppStore()

  const startService = async () => {
    try {
      await store.startService()
    } catch (error) {
      console.error('Failed to start service:', error)
      throw error
    }
  }

  const stopService = async () => {
    try {
      await store.stopService()
    } catch (error) {
      console.error('Failed to stop service:', error)
      throw error
    }
  }

  const restartService = async () => {
    try {
      await store.restartService()
    } catch (error) {
      console.error('Failed to restart service:', error)
      throw error
    }
  }

  return {
    startService,
    stopService,
    restartService
  }
}
