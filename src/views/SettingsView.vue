<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useAppStore } from '../stores/app'
import { useTranslation } from '../composables/useI18n'
import { Settings, Server, HardDrive, Save, RotateCcw, AlertCircle, CheckCircle, FolderOpen } from 'lucide-vue-next'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import { open } from '@tauri-apps/plugin-dialog'

const appStore = useAppStore()
const route = useRoute()
const { t } = useTranslation()
const isSaving = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error' | 'info'>('info')
const activeTab = ref('openlist')
const rcloneConfigJson = ref('')
const autoStartApp = ref(false)

const openlistCoreSettings = reactive({ ...appStore.settings.openlist })
const rcloneSettings = reactive({ ...appStore.settings.rclone })
const appSettings = reactive({ ...appStore.settings.app })
let originalOpenlistPort = openlistCoreSettings.port || 5244
let originalDataDir = openlistCoreSettings.data_dir

watch(autoStartApp, async newValue => {
  if (newValue) {
    await enable()
  } else {
    await disable()
  }
})

const tabs = computed(() => [
  {
    id: 'openlist',
    label: t('settings.tabs.openlist'),
    icon: Server,
    description: t('settings.service.subtitle')
  },
  {
    id: 'rclone',
    label: t('settings.tabs.rclone'),
    icon: HardDrive,
    description: t('settings.rclone.subtitle')
  },
  {
    id: 'app',
    label: t('settings.tabs.app'),
    icon: Settings,
    description: t('settings.app.subtitle')
  }
])

onMounted(async () => {
  autoStartApp.value = await isEnabled()
  const tabParam = route.query.tab as string
  if (tabParam && ['openlist', 'rclone', 'app'].includes(tabParam)) {
    activeTab.value = tabParam
  }

  if (!openlistCoreSettings.port) openlistCoreSettings.port = 5244
  if (!openlistCoreSettings.data_dir) openlistCoreSettings.data_dir = ''
  if (openlistCoreSettings.auto_launch === undefined) openlistCoreSettings.auto_launch = false
  if (openlistCoreSettings.ssl_enabled === undefined) openlistCoreSettings.ssl_enabled = false

  if (!rcloneSettings.config) rcloneSettings.config = {}

  rcloneConfigJson.value = JSON.stringify(rcloneSettings.config, null, 2)
  if (!appSettings.theme) appSettings.theme = 'light'

  if (appSettings.auto_update_enabled === undefined) appSettings.auto_update_enabled = true
  if (!appSettings.gh_proxy) appSettings.gh_proxy = ''
  if (appSettings.gh_proxy_api === undefined) appSettings.gh_proxy_api = false
  if (appSettings.open_links_in_browser === undefined) appSettings.open_links_in_browser = false
  originalOpenlistPort = openlistCoreSettings.port || 5244
  originalDataDir = openlistCoreSettings.data_dir
})

const hasUnsavedChanges = computed(() => {
  let rcloneConfigChanged = false
  try {
    const parsedConfig = JSON.parse(rcloneConfigJson.value)
    rcloneConfigChanged = JSON.stringify(parsedConfig) !== JSON.stringify(appStore.settings.rclone.config)
  } catch {
    rcloneConfigChanged = rcloneConfigJson.value !== JSON.stringify(appStore.settings.rclone.config, null, 2)
  }

  return (
    JSON.stringify(openlistCoreSettings) !== JSON.stringify(appStore.settings.openlist) ||
    JSON.stringify(rcloneSettings) !== JSON.stringify(appStore.settings.rclone) ||
    JSON.stringify(appSettings) !== JSON.stringify(appStore.settings.app) ||
    rcloneConfigChanged
  )
})

const handleSave = async () => {
  isSaving.value = true
  message.value = ''

  try {
    try {
      rcloneSettings.config = JSON.parse(rcloneConfigJson.value)
    } catch (error) {
      message.value = t('settings.rclone.config.invalidJson')
      messageType.value = 'error'
      isSaving.value = false
      return
    }

    appStore.settings.openlist = { ...openlistCoreSettings }
    appStore.settings.rclone = { ...rcloneSettings }
    appStore.settings.app = { ...appSettings }
    if (originalOpenlistPort !== openlistCoreSettings.port || originalDataDir !== openlistCoreSettings.data_dir) {
      await appStore.saveSettingsWithCoreUpdate()
    } else {
      await appStore.saveSettings()
    }

    originalOpenlistPort = openlistCoreSettings.port || 5244
    originalDataDir = openlistCoreSettings.data_dir

    message.value = t('settings.saved')
    messageType.value = 'success'
  } catch (error) {
    message.value = t('settings.saveFailed')
    messageType.value = 'error'
    console.error('Save settings error:', error)
  } finally {
    isSaving.value = false

    setTimeout(() => {
      message.value = ''
    }, 3000)
  }
}

const handleReset = async () => {
  if (!confirm(t('settings.confirmReset'))) {
    return
  }

  try {
    await appStore.resetSettings()
    Object.assign(openlistCoreSettings, appStore.settings.openlist)
    Object.assign(rcloneSettings, appStore.settings.rclone)
    Object.assign(appSettings, appStore.settings.app)

    rcloneConfigJson.value = JSON.stringify(rcloneSettings.config, null, 2)

    message.value = t('settings.resetSuccess')
    messageType.value = 'info'
  } catch (error) {
    message.value = t('settings.resetFailed')
    messageType.value = 'error'
  }
}

const handleSelectDataDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('settings.service.network.dataDir.selectTitle'),
      defaultPath: openlistCoreSettings.data_dir || undefined
    })

    if (selected && typeof selected === 'string') {
      openlistCoreSettings.data_dir = selected
    }
  } catch (error) {
    console.error('Failed to select directory:', error)
    message.value = t('settings.service.network.dataDir.selectError')
    messageType.value = 'error'
    setTimeout(() => {
      message.value = ''
    }, 3000)
  }
}
</script>

<template>
  <div class="settings-container">
    <div class="settings-header">
      <div class="header-content">
        <Settings :size="24" class="header-icon" />
        <div>
          <h1>{{ t('settings.title') }}</h1>
          <p>{{ t('settings.subtitle') }}</p>
        </div>
      </div>
      <div class="header-actions">
        <button @click="handleReset" class="btn btn-secondary" :title="t('settings.resetToDefaults')">
          <RotateCcw :size="16" />
          {{ t('common.reset') }}
        </button>
        <button @click="handleSave" :disabled="!hasUnsavedChanges || isSaving" class="btn btn-primary">
          <Save :size="16" />
          {{ isSaving ? t('common.saving') : t('settings.saveChanges') }}
        </button>
      </div>
    </div>

    <div v-if="message" class="message-banner" :class="messageType">
      <component :is="messageType === 'success' ? CheckCircle : AlertCircle" :size="16" />
      <span>{{ message }}</span>
      <button @click="message = ''" class="message-close">×</button>
    </div>

    <div class="tab-navigation">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="activeTab = tab.id"
        class="tab-button"
        :class="{ active: activeTab === tab.id }"
      >
        <component :is="tab.icon" :size="18" />
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <div class="settings-content">
      <div v-if="activeTab === 'openlist'" class="tab-content">
        <div class="settings-section">
          <h2>{{ t('settings.network.title') }}</h2>
          <p>{{ t('settings.network.subtitle') }}</p>

          <div class="form-grid">
            <div class="form-group">
              <label>{{ t('settings.service.network.port.label') }}</label>
              <input
                v-model.number="openlistCoreSettings.port"
                type="number"
                class="form-input"
                :placeholder="t('settings.service.network.port.placeholder')"
                min="1"
                max="65535"
              />
              <small>{{ t('settings.service.network.port.help') }}</small>
            </div>
            <div class="form-group">
              <label>{{ t('settings.service.network.dataDir.label') }}</label>
              <div class="input-group">
                <input
                  v-model="openlistCoreSettings.data_dir"
                  type="text"
                  class="form-input"
                  :placeholder="t('settings.service.network.dataDir.placeholder')"
                />
                <button
                  type="button"
                  @click="handleSelectDataDir"
                  class="input-addon-btn"
                  :title="t('settings.service.network.dataDir.selectTitle')"
                >
                  <FolderOpen :size="16" />
                </button>
              </div>
              <small>{{ t('settings.service.network.dataDir.help') }}</small>
            </div>

            <div class="form-group">
              <label class="switch-label">
                <input v-model="openlistCoreSettings.ssl_enabled" type="checkbox" class="switch-input" />
                <span class="switch-slider"></span>
                <div class="switch-content">
                  <span class="switch-title">{{ t('settings.service.network.ssl.title') }}</span>
                  <span class="switch-description">{{ t('settings.service.network.ssl.description') }}</span>
                </div>
              </label>
            </div>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.startup.title') }}</h2>
          <p>{{ t('settings.startup.subtitle') }}</p>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="openlistCoreSettings.auto_launch" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.service.startup.autoLaunch.title') }}</span>
                <span class="switch-description">{{ t('settings.service.startup.autoLaunch.description') }}</span>
              </div>
            </label>
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'rclone'" class="tab-content">
        <div class="settings-section">
          <h2>{{ t('settings.rclone.config.title') }}</h2>
          <p>{{ t('settings.rclone.config.subtitle') }}</p>

          <div class="form-group">
            <label>{{ t('settings.rclone.config.label') }}</label>
            <textarea
              v-model="rcloneConfigJson"
              class="form-textarea"
              placeholder='{ "remote1": { "type": "s3", "provider": "AWS" } }'
              rows="10"
              readonly
            ></textarea>
            <small>{{ t('settings.rclone.config.tips') }}</small>
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'app'" class="tab-content">
        <div class="settings-section">
          <h2>{{ t('settings.app.theme.title') }}</h2>
          <p>{{ t('settings.app.theme.subtitle') }}</p>

          <div class="form-grid">
            <div class="form-group">
              <label>{{ t('settings.theme.title') }}</label>
              <select
                v-model="appSettings.theme"
                @change="appStore.setTheme(appSettings.theme || 'light')"
                class="form-input"
              >
                <option value="light">{{ t('settings.app.theme.light') }}</option>
                <option value="dark">{{ t('settings.app.theme.dark') }}</option>
                <option value="auto">{{ t('settings.app.theme.auto') }}</option>
              </select>
              <small>{{ t('settings.app.theme.autoDesc') }}</small>
            </div>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.ghProxy.title') }}</h2>
          <p>{{ t('settings.app.ghProxy.subtitle') }}</p>

          <div class="form-grid">
            <div class="form-group">
              <label>{{ t('settings.app.ghProxy.label') }}</label>
              <input
                v-model="appSettings.gh_proxy"
                type="text"
                class="form-input"
                :placeholder="t('settings.app.ghProxy.placeholder')"
              />
              <small>{{ t('settings.app.ghProxy.help') }}</small>
            </div>
          </div>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="appSettings.gh_proxy_api" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.app.ghProxy.api.title') }}</span>
                <span class="switch-description">{{ t('settings.app.ghProxy.api.description') }}</span>
              </div>
            </label>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.autoStartApp.title') }}</h2>
          <p>{{ t('settings.app.autoStartApp.subtitle') }}</p>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="autoStartApp" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.app.autoStartApp.title') }}</span>
                <span class="switch-description">{{ t('settings.app.autoStartApp.description') }}</span>
              </div>
            </label>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.updates.title') }}</h2>
          <p>{{ t('settings.app.updates.subtitle') }}</p>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="appSettings.auto_update_enabled" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.app.updates.autoCheck.title') }}</span>
                <span class="switch-description">{{ t('settings.app.updates.autoCheck.description') }}</span>
              </div>
            </label>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.links.title') }}</h2>
          <p>{{ t('settings.app.links.subtitle') }}</p>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="appSettings.open_links_in_browser" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.app.links.openInBrowser.title') }}</span>
                <span class="switch-description">{{ t('settings.app.links.openInBrowser.description') }}</span>
              </div>
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./css/SettingsView.css"></style>
