import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { appDataDir, join } from '@tauri-apps/api/path'

export class TauriAPI {
  // --- OpenList Core management ---
  static core = {
    create: (): Promise<ProcessInfo> => invoke('create_openlist_core_process'),
    start: (): Promise<ProcessInfo> => invoke('start_openlist_core'),
    stop: (): Promise<ProcessInfo> => invoke('stop_openlist_core'),
    restart: (): Promise<ProcessInfo> => invoke('restart_openlist_core'),
    getStatus: (): Promise<OpenListCoreStatus> => invoke('get_openlist_core_status'),
    getProcessStatus: (): Promise<ProcessInfo> => invoke('get_openlist_core_process_status'),
  }

  // --- Rclone management ---
  static rclone = {
    // Check if rclone binary is available
    isAvailable: (): Promise<boolean> => invoke('check_rclone_available'),

    // Remote configuration management (direct file-based)
    remotes: {
      list: (): Promise<string[]> => invoke('rclone_list_remotes'),
      create: (name: string, type: string, config: RcloneWebdavConfig): Promise<boolean> =>
        invoke('rclone_create_remote', { name, type, config }),
      update: (name: string, type: string, config: RcloneWebdavConfig): Promise<boolean> =>
        invoke('rclone_update_remote', { name, type, config }),
      delete: (name: string): Promise<boolean> => invoke('rclone_delete_remote', { name }),
      listConfig: (t: string): Promise<IRemoteConfig> => invoke('rclone_list_config', { remoteType: t }),
    },

    // Mount process management
    mounts: {
      list: (): Promise<RcloneMountInfo[]> => invoke('get_mount_info_list'),
      check: (id: string, mp: string): Promise<boolean> => invoke('check_mount_status', { id, mountPoint: mp }),
      mount: (cfg: MountProcessInput): Promise<ProcessInfo> =>
        invoke('create_rclone_mount_remote_process', { config: cfg }),
      unmount: (name: string): Promise<boolean> => invoke('unmount_remote', { name }),
    },
  }

  // -- File management ---
  static files = {
    list: (path: string): Promise<FileItem[]> => invoke('list_files', { path }),
    open: (path: string): Promise<boolean> => invoke('open_file', { path }),
    folder: (path: string): Promise<boolean> => invoke('open_folder', { path }),
    urlInBrowser: (url: string): Promise<boolean> => invoke('open_url_in_browser', { url }),
    openOpenListDataDir: (): Promise<boolean> => invoke('open_openlist_data_dir'),
    openLogsDirectory: (): Promise<boolean> => invoke('open_logs_directory'),
    openRcloneConfigFile: (): Promise<boolean> => invoke('open_rclone_config_file'),
    openSettingsFile: (): Promise<boolean> => invoke('open_settings_file'),
  }

  // --- Settings management ---
  static settings = {
    load: (): Promise<MergedSettings | null> => invoke('load_settings'),
    save: (s: MergedSettings): Promise<boolean> => invoke('save_settings', { settings: s }),
    saveAndRestart: (s: MergedSettings): Promise<boolean> => invoke('save_settings_and_restart', { settings: s }),
    reset: (): Promise<MergedSettings | null> => invoke('reset_settings'),
  }

  // --- Logs management ---
  static logs = {
    get: (src?: 'openlist' | 'rclone' | 'app' | 'openlist_core' | 'all'): Promise<string[]> =>
      invoke('get_logs', { source: src }),
    clear: (src?: 'openlist' | 'rclone' | 'app' | 'openlist_core' | 'all'): Promise<boolean> =>
      invoke('clear_logs', { source: src }),
    adminPassword: (): Promise<string> => invoke('get_admin_password'),
    resetAdminPassword: (): Promise<string> => invoke('reset_admin_password'),
    setAdminPassword: (password: string): Promise<string> => invoke('set_admin_password', { password }),
  }

  // --- Binary management ---
  static bin = {
    version: (name: 'openlist' | 'rclone'): Promise<string> => invoke('get_binary_version', { binaryName: name }),
    availableVersions: (tool: string, force: boolean): Promise<string[]> =>
      invoke('get_available_versions', { tool, force }),
    updateVersion: (tool: string, version: string): Promise<string> => invoke('update_tool_version', { tool, version }),
  }

  // --- Utility functions ---
  static util = {
    defaultDataDir: (): Promise<string> => appDataDir().then(d => join(d, 'openlist-desktop')),
    defaultConfig: (): Promise<string> => appDataDir().then(d => join(d, 'openlist-desktop', 'rclone.conf')),
    selectDirectory: (title: string): Promise<string | null> => invoke('select_directory', { title }),
  }

  // --- Tray management ---
  static tray = {
    update: (r: boolean): Promise<void> => invoke('update_tray_menu', { serviceRunning: r }),
  }

  // --- macOS Dock management ---
  static dock = {
    setVisibility: (visible: boolean): Promise<boolean> => invoke('set_dock_icon_visibility', { visible }),
  }

  // --- Firewall management ---
  static firewall = {
    check: (): Promise<boolean> => invoke('check_firewall_rule'),
    add: (): Promise<boolean> => invoke('add_firewall_rule'),
    remove: (): Promise<boolean> => invoke('remove_firewall_rule'),
  }

  // --- Update management ---
  static updater = {
    check: (): Promise<UpdateCheck> => invoke('check_for_updates'),
    download: (url: string, name: string): Promise<string> =>
      invoke('download_update', { assetUrl: url, assetName: name }),
    installAndRestart: (path: string): Promise<void> => invoke('install_update_and_restart', { installerPath: path }),
    currentVersion: (): Promise<string> => invoke('get_current_version'),
    setAutoCheck: (e: boolean): Promise<void> => invoke('set_auto_check_enabled', { enabled: e }),
    isAutoCheckEnabled: (): Promise<boolean> => invoke('is_auto_check_enabled'),
    onBackgroundUpdate: (cb: (u: UpdateCheck) => void) =>
      listen('background-update-available', e => cb(e.payload as UpdateCheck)),
    onDownloadProgress: (cb: (p: DownloadProgress) => void) =>
      listen('download-progress', e => cb(e.payload as DownloadProgress)),
    onInstallStarted: (cb: () => void) => listen('update-install-started', () => cb()),
    onInstallError: (cb: (err: string) => void) => listen('update-install-error', e => cb(e.payload as string)),
    onAppQuit: (cb: () => void) => listen('quit-app', () => cb()),
  }
}
