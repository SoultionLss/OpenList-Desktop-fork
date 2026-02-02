export const DEFAULT_CONFIG = {
  openlistCore: {
    port: 5244,
    data_dir: '',
    auto_launch: false,
    ssl_enabled: false,
    binary_path: '',
  },
  rclone: {
    config: {},
    binary_path: '',
    rclone_conf_path: '',
  },
  app: {
    theme: 'light',
    auto_update_enabled: true,
    gh_proxy: '',
    gh_proxy_api: false,
    open_links_in_browser: false,
    show_window_on_startup: true,
    hide_dock_icon: false,
    admin_password: '',
  },
}
