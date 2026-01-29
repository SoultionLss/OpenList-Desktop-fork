import { useAppStore } from '../stores/app'

export const useCoreActions = () => {
  const appStore = useAppStore()

  const startOpenListCore = async () => {
    try {
      await appStore.startOpenListCore()
    } catch (error) {
      console.error('Failed to start service:', error)
      throw error
    }
  }

  const stopOpenListCore = async () => {
    try {
      await appStore.stopOpenListCore()
    } catch (error) {
      console.error('Failed to stop service:', error)
      throw error
    }
  }

  const restartOpenListCore = async () => {
    try {
      await appStore.restartOpenListCore()
    } catch (error) {
      console.error('Failed to restart service:', error)
      throw error
    }
  }

  return {
    startOpenListCore,
    stopOpenListCore,
    restartOpenListCore,
  }
}
