import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { appDataDir, join } from '@tauri-apps/api/path'

const call = <T>(cmd: string, args?: any): Promise<T> => invoke(cmd, args)

export class TauriAPI {
  // --- OpenList Core management ---
  static core = {
    create: (): Promise<ProcessInfo> => call('create_openlist_core_process'),
    start: (): Promise<ProcessInfo> => call('start_openlist_core'),
    stop: (): Promise<ProcessInfo> => call('stop_openlist_core'),
    restart: (): Promise<ProcessInfo> => call('restart_openlist_core'),
    getStatus: (): Promise<OpenListCoreStatus> => call('get_openlist_core_status'),
    getProcessStatus: (): Promise<ProcessInfo> => call('get_openlist_core_process_status'),
    getLogs: (lines?: number): Promise<string[]> => call('get_openlist_core_logs', { lines }),
  }

  // --- Rclone management ---
  static rclone = {
    // Check if rclone binary is available
    isAvailable: (): Promise<boolean> => call('check_rclone_available'),

    // Remote configuration management (direct file-based)
    remotes: {
      list: (): Promise<string[]> => call('rclone_list_remotes'),
      create: (name: string, type: string, config: RcloneWebdavConfig): Promise<boolean> =>
        call('rclone_create_remote', { name, type, config }),
      update: (name: string, type: string, config: RcloneWebdavConfig): Promise<boolean> =>
        call('rclone_update_remote', { name, type, config }),
      delete: (name: string): Promise<boolean> => call('rclone_delete_remote', { name }),
      listConfig: (t: string): Promise<IRemoteConfig> => call('rclone_list_config', { remoteType: t }),
    },

    // Mount process management
    mounts: {
      list: (): Promise<RcloneMountInfo[]> => call('get_mount_info_list'),
      check: (mp: string): Promise<boolean> => call('check_mount_status', { mountPoint: mp }),
      createProcess: (cfg: MountProcessInput): Promise<ProcessInfo> =>
        call('create_rclone_mount_remote_process', { config: cfg }),
      startProcess: (processId: string): Promise<ProcessInfo> => call('start_mount_process', { processId }),
      stopProcess: (processId: string): Promise<ProcessInfo> => call('stop_mount_process', { processId }),
      unmount: (name: string): Promise<boolean> => call('unmount_remote', { name }),
      getLogs: (processId: string, lines?: number): Promise<string[]> =>
        call('get_mount_process_logs', { processId, lines }),
    },
  }

  // -- File management ---
  static files = {
    list: (path: string): Promise<FileItem[]> => call('list_files', { path }),
    open: (path: string): Promise<boolean> => call('open_file', { path }),
    folder: (path: string): Promise<boolean> => call('open_folder', { path }),
    urlInBrowser: (url: string): Promise<boolean> => call('open_url_in_browser', { url }),
    openOpenListDataDir: (): Promise<boolean> => call('open_openlist_data_dir'),
    openLogsDirectory: (): Promise<boolean> => call('open_logs_directory'),
    openRcloneConfigFile: (): Promise<boolean> => call('open_rclone_config_file'),
    openSettingsFile: (): Promise<boolean> => call('open_settings_file'),
  }

  // --- Settings management ---
  static settings = {
    load: (): Promise<MergedSettings | null> => call('load_settings'),
    save: (s: MergedSettings): Promise<boolean> => call('save_settings', { settings: s }),
    saveWithUpdatePort: (s: MergedSettings): Promise<boolean> =>
      call('save_settings_with_update_port', { settings: s }),
    reset: (): Promise<MergedSettings | null> => call('reset_settings'),
  }

  // --- Logs management ---
  static logs = {
    get: (src?: 'openlist' | 'rclone' | 'app' | 'openlist_core' | 'all'): Promise<string[]> =>
      call('get_logs', { source: src }),
    clear: (src?: 'openlist' | 'rclone' | 'app' | 'openlist_core' | 'all'): Promise<boolean> =>
      call('clear_logs', { source: src }),
    adminPassword: (): Promise<string> => call('get_admin_password'),
    resetAdminPassword: (): Promise<string> => call('reset_admin_password'),
    setAdminPassword: (password: string): Promise<string> => call('set_admin_password', { password }),
  }

  // --- Binary management ---
  static bin = {
    version: (name: 'openlist' | 'rclone'): Promise<string> => call('get_binary_version', { binaryName: name }),
    availableVersions: (tool: string): Promise<string[]> => call('get_available_versions', { tool }),
    updateVersion: (tool: string, version: string): Promise<string> => call('update_tool_version', { tool, version }),
  }

  // --- Utility functions ---
  static util = {
    defaultDataDir: (): Promise<string> => appDataDir().then(d => join(d, 'openlist-desktop')),
    defaultConfig: (): Promise<string> => appDataDir().then(d => join(d, 'openlist-desktop', 'rclone.conf')),
    selectDirectory: (title: string): Promise<string | null> => call('select_directory', { title }),
  }

  // --- Tray management ---
  static tray = {
    update: (r: boolean): Promise<void> => call('update_tray_menu', { serviceRunning: r }),
  }

  // --- macOS Dock management ---
  static dock = {
    setVisibility: (visible: boolean): Promise<boolean> => call('set_dock_icon_visibility', { visible }),
  }

  // --- Firewall management ---
  static firewall = {
    check: (): Promise<boolean> => call('check_firewall_rule'),
    add: (): Promise<boolean> => call('add_firewall_rule'),
    remove: (): Promise<boolean> => call('remove_firewall_rule'),
  }

  // --- Update management ---
  static updater = {
    check: (): Promise<UpdateCheck> => call('check_for_updates'),
    download: (url: string, name: string): Promise<string> =>
      call('download_update', { assetUrl: url, assetName: name }),
    installAndRestart: (path: string): Promise<void> => call('install_update_and_restart', { installerPath: path }),
    currentVersion: (): Promise<string> => call('get_current_version'),
    setAutoCheck: (e: boolean): Promise<void> => call('set_auto_check_enabled', { enabled: e }),
    isAutoCheckEnabled: (): Promise<boolean> => call('is_auto_check_enabled'),
    onBackgroundUpdate: (cb: (u: UpdateCheck) => void) =>
      listen('background-update-available', e => cb(e.payload as UpdateCheck)),
    onDownloadProgress: (cb: (p: DownloadProgress) => void) =>
      listen('download-progress', e => cb(e.payload as DownloadProgress)),
    onInstallStarted: (cb: () => void) => listen('update-install-started', () => cb()),
    onInstallError: (cb: (err: string) => void) => listen('update-install-error', e => cb(e.payload as string)),
    onAppQuit: (cb: () => void) => listen('quit-app', () => cb()),
  }
}
