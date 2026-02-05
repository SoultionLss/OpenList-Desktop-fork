import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { TauriAPI } from '../api/tauri'

type ActionFn<T = any> = () => Promise<T>

export const useAppStore = defineStore('app', () => {
  const settings = ref<MergedSettings>({
    openlist: { port: 5244, data_dir: '', auto_launch: false, ssl_enabled: false, binary_path: undefined },
    rclone: { binary_path: undefined, rclone_conf_path: undefined, mount_config: {} },
    app: {
      theme: 'light',
      auto_update_enabled: true,
      gh_proxy: '',
      gh_proxy_api: false,
      open_links_in_browser: false,
      admin_password: undefined,
      show_window_on_startup: true,
    },
  })
  const openlistCoreStatus = ref<OpenListCoreStatus>({ running: false })
  const remoteConfigs = ref<IRemoteConfig>({})
  const mountInfos = ref<RcloneMountInfo[]>([])
  const logs = ref<string[]>([])
  const files = ref<FileItem[]>([])
  const currentPath = ref('/')
  const loading = ref(false)
  const error = ref<string | undefined>()
  const updateAvailable = ref(false)
  const updateCheck = ref<UpdateCheck | null>(null)
  const openlistProcessId = ref<string | undefined>(undefined)

  const defaultRcloneFormConfig: RcloneFormConfig = {
    name: '',
    type: 'webdav',
    url: '',
    vendor: '',
    user: '',
    pass: '',
    mountPoint: '',
    volumeName: '',
    extraFlags: [],
    autoMount: false,
  }

  // Computed
  const mountedConfigs = computed(() => mountInfos.value.filter(mount => mount.status === 'mounted'))

  const fullRcloneConfigs = computed<RcloneFormConfig[]>(() => {
    const result: RcloneFormConfig[] = []
    for (const [key, config] of Object.entries(remoteConfigs.value)) {
      let newConfig
      if (settings.value.rclone.mount_config[key]) {
        newConfig = {
          name: key,
          type: 'webdav',
          url: config.url,
          vendor: config.vendor,
          user: config.user,
          pass: config.pass,
          mountPoint: settings.value.rclone.mount_config[key].mountPoint || '',
          volumeName: settings.value.rclone.mount_config[key].volumeName || '',
          extraFlags: settings.value.rclone.mount_config[key].extraFlags || [],
          autoMount: settings.value.rclone.mount_config[key].autoMount ?? false,
        }
      } else {
        newConfig = {
          ...defaultRcloneFormConfig,
          name: key,
          url: config.url,
          vendor: config.vendor,
          user: config.user,
          pass: config.pass,
        } as RcloneFormConfig
      }
      result.push(newConfig)
      settings.value.rclone.mount_config[key] = newConfig
    }
    return result
  })
  const isCoreRunning = computed(() => openlistCoreStatus.value.running)
  const openListCoreUrl = computed(() => {
    const protocol = settings.value.openlist.ssl_enabled ? 'https' : 'http'
    return `${protocol}://localhost:${settings.value.openlist.port}`
  })

  // Helper
  async function withLoading<T>(fn: ActionFn<T>, msg: string): Promise<T> {
    loading.value = true
    try {
      return await fn()
    } catch (e: any) {
      error.value = msg
      console.error(msg, e)
      throw e
    } finally {
      loading.value = false
    }
  }
  // Settings
  const loadSettings = () =>
    withLoading(async () => {
      const res = await TauriAPI.settings.load()
      if (res) settings.value = res
      applyTheme(settings.value.app.theme || 'light')
    }, 'Failed to load settings')

  const saveSettings = () => withLoading(() => TauriAPI.settings.save(settings.value), 'Failed to save settings')

  async function saveAndRestart(): Promise<boolean> {
    try {
      await TauriAPI.settings.saveAndRestart(settings.value)
      return true
    } catch (err) {
      error.value = 'Failed to save settings'
      console.error('Failed to save settings:', err)
      return false
    }
  }

  const resetSettings = () =>
    withLoading(async () => {
      const res = await TauriAPI.settings.reset()
      if (res) settings.value = res
    }, 'Failed to reset settings')

  async function loadMountInfos() {
    try {
      mountInfos.value = await TauriAPI.rclone.mounts.list()
    } catch (err: any) {
      error.value = 'Failed to load mount information'
      console.error('Failed to load mount infos:', err)
    }
  }

  async function createRemoteConfig(name: string, type: string, config: RcloneFormConfig) {
    try {
      loading.value = true
      const fullConfig = {
        name,
        type,
        url: config.url,
        vendor: config.vendor || '',
        user: config.user,
        pass: config.pass,
        mountPoint: config.mountPoint || '',
        volumeName: config.volumeName || '',
        extraFlags: config.extraFlags || [],
        autoMount: config.autoMount ?? false,
      }
      const createdConfig: RcloneWebdavConfig = {
        url: fullConfig.url,
        vendor: fullConfig.vendor || '',
        user: fullConfig.user,
        pass: fullConfig.pass,
      }
      const result = await TauriAPI.rclone.remotes.create(name, type, createdConfig)
      if (!result) {
        throw new Error('Failed to create remote configuration')
      }
      settings.value.rclone.mount_config[name] = fullConfig
      await loadRemoteConfigs()
      await saveSettings()
      return true
    } catch (err: any) {
      error.value = 'Failed to create remote configuration'
      console.error('Failed to create remote config:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateRemoteConfig(name: string, type: string, config: RcloneFormConfig) {
    try {
      loading.value = true
      const fullConfig = {
        name: config.name,
        type,
        url: config.url,
        vendor: config.vendor || undefined,
        user: config.user,
        pass: config.pass,
        mountPoint: config.mountPoint || undefined,
        volumeName: config.volumeName || undefined,
        extraFlags: config.extraFlags || [],
        autoMount: config.autoMount ?? false,
      }
      const updatedConfig: RcloneWebdavConfig = {
        url: fullConfig.url,
        vendor: fullConfig.vendor || undefined,
        user: fullConfig.user,
        pass: fullConfig.pass,
      }
      if (name !== config.name) {
        const result = createRemoteConfig(config.name, type, config)
        if (!result) {
          throw new Error('Failed to create remote configuration')
        }
        const deleteResult = await TauriAPI.rclone.remotes.delete(name)
        if (!deleteResult) {
          throw new Error('Failed to delete old remote configuration')
        }
      } else {
        const result = await TauriAPI.rclone.remotes.update(name, type, updatedConfig)
        if (!result) {
          throw new Error('Failed to update remote configuration')
        }
        settings.value.rclone.mount_config[config.name] = fullConfig
      }
      await loadRemoteConfigs()
      if (name !== config.name && settings.value.rclone.mount_config[name]) {
        delete settings.value.rclone.mount_config[name]
      }
      await saveSettings()
      await loadMountInfos()
      return true
    } catch (err: any) {
      error.value = 'Failed to update remote configuration'
      console.error('Failed to update remote config:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteRemoteConfig(name: string) {
    try {
      const existingMount = mountInfos.value.find(m => m.name === name)
      if (existingMount && existingMount.status === 'mounted') {
        await unmountRemote(name)
      }
      await TauriAPI.rclone.remotes.delete(name)
      await loadRemoteConfigs()
      if (settings.value.rclone.mount_config[name]) {
        delete settings.value.rclone.mount_config[name]
        await saveSettings()
      }
      await loadMountInfos()
      return true
    } catch (err: any) {
      error.value = 'Failed to delete remote configuration'
      console.error('Failed to delete remote config:', err)
      throw err
    }
  }

  async function loadRemoteConfigs() {
    try {
      loading.value = true
      remoteConfigs.value = await TauriAPI.rclone.remotes.listConfig('webdav')
    } catch (err: any) {
      error.value = 'Failed to load remote configurations'
      console.error('Failed to load remote configs:', err)
    } finally {
      loading.value = false
    }
  }

  function getFullRcloneConfigs(name?: string): RcloneFormConfig[] {
    return name ? fullRcloneConfigs.value.filter(c => c.name === name) : fullRcloneConfigs.value
  }

  async function mountRemote(name: string) {
    try {
      const config = settings.value.rclone.mount_config[name] as RcloneFormConfig | undefined
      if (!config) {
        throw new Error(`No configuration found for remote: ${name}`)
      }

      if (!config.mountPoint) {
        throw new Error(`Mount point is not set for remote: ${name}`)
      }
      const id = `rclone_mount_${name}_process`
      const mountArgs = [
        `${config.name}:${config.volumeName || ''}`,
        config.mountPoint || '',
        ...(config.extraFlags || []),
      ]
      const createRemoteConfig: MountProcessInput = {
        id,
        name: id,
        args: mountArgs,
      }
      const createResponse = await TauriAPI.rclone.mounts.mount(createRemoteConfig)
      if (!createResponse || !createResponse.id) {
        throw new Error('Failed to create mount process')
      }
      await new Promise(resolve => setTimeout(resolve, 3000))
      await loadMountInfos()
    } catch (err: any) {
      error.value = `Failed to mount remote ${name}: ${formatError(err)}`
      console.error('Failed to mount remote:', err)
      throw err
    }
  }

  async function stopMountProcess(name: string) {
    try {
      await TauriAPI.rclone.mounts.unmount(name)
      await loadMountInfos()
    } catch (err: any) {
      error.value = `Failed to stop mount process for remote ${name}: ${formatError(err)}`
      console.error('Failed to stop mount process:', err)
      throw err
    }
  }

  async function unmountRemote(name: string) {
    try {
      await TauriAPI.rclone.mounts.unmount(name)
      await loadMountInfos()
    } catch (err: any) {
      error.value = `Failed to unmount remote ${name}: ${formatError(err)}`
      console.error('Failed to unmount remote:', err)
      throw err
    }
  }

  async function startOpenListCore() {
    try {
      loading.value = true
      const createResponse = await TauriAPI.core.create()
      if (!createResponse || !createResponse.id) {
        throw new Error('Invalid response from TauriAPI.core.create: missing process ID')
      }
      openlistProcessId.value = createResponse.id
      await refreshOpenListCoreStatus()

      await TauriAPI.tray.update(openlistCoreStatus.value.running)
    } catch (err: any) {
      openlistCoreStatus.value = { running: false }
      let errorMessage = 'Failed to start service'
      const formattedError = formatError(err)
      if (formattedError) {
        errorMessage += `: ${formattedError}`
      }
      error.value = errorMessage
      console.error('Failed to start service:', err)
      await safeUpdateTrayMenu(false)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function stopOpenListCore() {
    try {
      loading.value = true
      await TauriAPI.core.stop()
      openlistCoreStatus.value = { running: false }
      await TauriAPI.tray.update(false)
    } catch (err: any) {
      const errorMessage = `Failed to stop service: ${formatError(err)}`
      error.value = errorMessage
      console.error('Failed to stop service:', err)
      try {
        await refreshOpenListCoreStatus()
      } catch (refreshErr) {
        console.error('Failed to refresh service status after stop failure:', refreshErr)
      }
      throw err
    } finally {
      loading.value = false
    }
  }

  async function restartOpenListCore() {
    try {
      loading.value = true
      await TauriAPI.core.restart()
      await refreshOpenListCoreStatus()
      await TauriAPI.tray.update(openlistCoreStatus.value.running)
    } catch (err: any) {
      const errorMessage = `Failed to restart core: ${formatError(err)}`
      error.value = errorMessage
      console.error('Failed to restart core:', err)
      try {
        await refreshOpenListCoreStatus()
        await safeUpdateTrayMenu(openlistCoreStatus.value.running)
      } catch (refreshErr) {
        console.error('Failed to refresh core status after restart failure:', refreshErr)
      }
      throw err
    } finally {
      loading.value = false
    }
  }

  async function refreshOpenListCoreStatus() {
    try {
      const status = await TauriAPI.core.getStatus()
      const statusChanged = openlistCoreStatus.value.running !== status.running
      openlistCoreStatus.value = status
      if (statusChanged) {
        await TauriAPI.tray.update(status.running)
      }
    } catch (_err) {
      const wasRunning = openlistCoreStatus.value.running
      openlistCoreStatus.value = { running: false }
      if (wasRunning) {
        await TauriAPI.tray.update(false)
      }
    }
  }

  async function loadLogs(source?: 'openlist' | 'rclone' | 'app' | 'all') {
    try {
      source = source || 'openlist'
      const logEntries = await TauriAPI.logs.get(source)
      logs.value = logEntries
    } catch (err) {
      console.error('Failed to load logs:', err)
    }
  }

  async function clearLogs(source?: 'openlist' | 'rclone' | 'app' | 'all') {
    try {
      loading.value = true
      source = source || 'openlist'
      const result = await TauriAPI.logs.clear(source)
      if (result) {
        logs.value = []
      } else {
        throw new Error('Failed to clear logs - backend returned false')
      }
    } catch (err) {
      error.value = 'Failed to clear logs'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function openFile(path: string) {
    try {
      await TauriAPI.files.open(path)
    } catch (err) {
      error.value = 'Failed to open file'
      console.error('Failed to open file:', err)
    }
  }

  async function openFolder(path: string) {
    await TauriAPI.files.folder(path)
  }

  async function openLogsDirectory() {
    await TauriAPI.files.openLogsDirectory()
  }

  async function openOpenListDataDir() {
    await TauriAPI.files.openOpenListDataDir()
  }

  async function openRcloneConfigFile() {
    await TauriAPI.files.openRcloneConfigFile()
  }

  async function openSettingsFile() {
    await TauriAPI.files.openSettingsFile()
  }

  async function selectDirectory(title: string): Promise<string | null> {
    try {
      return await TauriAPI.util.selectDirectory(title)
    } catch (err) {
      console.error('Failed to select directory:', err)
      return null
    }
  }

  function clearError() {
    error.value = undefined
  }

  function formatError(err: any): string {
    if (err?.message) {
      return err.message
    } else if (typeof err === 'string') {
      return err
    } else if (err?.toString) {
      return err.toString()
    }
    return 'Unknown error occurred'
  }

  async function safeUpdateTrayMenu(running: boolean) {
    try {
      await TauriAPI.tray.update(running)
    } catch (err) {
      console.warn('Failed to update tray menu:', err)
    }
  }

  async function autoStartCoreIfEnabled() {
    try {
      if (settings.value.openlist.auto_launch) {
        await startOpenListCore()
      }
    } catch (err) {
      console.warn('Failed to auto-start core:', err)
    }
  }

  function applyTheme(theme: string) {
    const root = document.documentElement
    root.classList.remove('light', 'dark', 'auto')

    if (theme === 'auto') {
      root.classList.add('auto')
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      root.classList.add(prefersDark ? 'dark' : 'light')
      root.setAttribute('data-theme', prefersDark ? 'dark' : 'light')

      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      mediaQuery.addEventListener('change', e => {
        if (settings.value.app.theme === 'auto') {
          root.classList.remove('light', 'dark')
          root.classList.add(e.matches ? 'dark' : 'light')
          root.setAttribute('data-theme', e.matches ? 'dark' : 'light')
        }
      })
    } else {
      root.classList.add(theme)
      root.setAttribute('data-theme', theme)
    }
  }

  function setTheme(theme: 'light' | 'dark' | 'auto') {
    settings.value.app.theme = theme
    applyTheme(theme)
    saveSettings().catch(console.error)
  }

  function toggleTheme() {
    const currentTheme = settings.value.app.theme || 'light'
    const themes = ['light', 'dark', 'auto'] as const
    const currentIndex = themes.indexOf(currentTheme as any)
    const nextTheme = themes[(currentIndex + 1) % themes.length]
    setTheme(nextTheme)
  }

  async function resetAdminPassword(): Promise<string | null> {
    try {
      const newPassword = await TauriAPI.logs.resetAdminPassword()
      if (newPassword) {
        settings.value.app.admin_password = newPassword
        await saveSettings()
      }
      return newPassword
    } catch (err) {
      console.error('Failed to reset admin password:', err)
      return null
    }
  }

  async function setAdminPassword(password: string): Promise<boolean> {
    try {
      await TauriAPI.logs.setAdminPassword(password)
      settings.value.app.admin_password = password
      await saveSettings()
      return true
    } catch (err) {
      console.error('Failed to set admin password:', err)
      return false
    }
  }

  // Update management functions
  function setUpdateAvailable(available: boolean, updateInfo?: UpdateCheck) {
    updateAvailable.value = available
    updateCheck.value = updateInfo || null
  }

  function clearUpdateStatus() {
    updateAvailable.value = false
    updateCheck.value = null
  }

  async function init() {
    try {
      await loadSettings()
      await refreshOpenListCoreStatus()
      loadLogs()
      autoStartCoreIfEnabled()
      await loadRemoteConfigs()
      await loadMountInfos()
    } catch (err) {
      console.error('Application initialization failed:', err)
      throw err
    }
  }

  return {
    remoteConfigs,
    mountInfos,
    mountedConfigs,
    mountRemote,
    unmountRemote,
    createRemoteConfig,
    updateRemoteConfig,
    loadRemoteConfigs,
    defaultRcloneFormConfig,
    loadMountInfos,
    stopMountProcess,
    deleteRemoteConfig,
    getFullRcloneConfigs,
    fullRcloneConfigs,

    settings,
    openlistCoreStatus,
    logs,
    files,
    currentPath,
    loading,
    error,
    updateAvailable,
    updateCheck,

    isCoreRunning,
    openListCoreUrl,

    loadSettings,
    saveSettings,
    saveAndRestart,
    resetSettings,

    startOpenListCore,
    stopOpenListCore,
    restartOpenListCore,
    refreshOpenListCoreStatus,
    loadLogs,
    clearLogs,
    openFile,
    openFolder,
    openLogsDirectory,
    openOpenListDataDir,
    openRcloneConfigFile,
    openSettingsFile,
    selectDirectory,
    clearError,
    init,
    resetAdminPassword,
    setAdminPassword,

    setTheme,
    toggleTheme,
    applyTheme,

    setUpdateAvailable,
    clearUpdateStatus,
  }
})
