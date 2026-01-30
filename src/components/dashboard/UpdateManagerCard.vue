<!-- eslint-disable vue/no-v-html -->
<template>
  <div :title="t('update.title')" class="w-full h-full overflow-auto flex items-center justify-center">
    <div class="w-full h-full flex flex-col gap-6">
      <div v-if="error" class="error-state">
        <div class="error-content">
          <AlertCircle :size="16" />
          <span>{{ error }}</span>
        </div>
        <button class="clear-error-btn" @click="clearError">×</button>
      </div>
      <SettingSection :icon="RefreshCw" :title="t('update.title')">
        <CustomNavCard noarrow :icon="RefreshCw" :title="t('update.currentVersion')">
          <template #description>
            <div class="flex items-center gap-2">
              <span class="rounded-md bg-accent/30 px-2 py-0.5 text-sm font-semibold text-white"
                >v{{ currentVersion }}</span
              >
            </div>
          </template>
          <template #extra>
            <CustomButton
              :icon="RefreshCw"
              :text="checking ? t('update.checking') : t('update.checkForUpdates')"
              type="secondary"
              :disabled="checking || downloading || installing"
              @click="checkForUpdates"
            />
          </template>
        </CustomNavCard>

        <SettingCard p1 class="flex items-center">
          <CustomSwitch
            v-model="autoCheckEnabled"
            no-border
            small
            :title="t('update.autoCheck')"
            @change="toggleAutoCheck"
          />
        </SettingCard>
      </SettingSection>

      <div v-if="!updateCheck?.hasUpdate && lastChecked && !checking && !error" class="no-updates">
        <CheckCircle :size="24" class="check-icon" />
        <div class="no-updates-text">
          <h4>{{ t('update.upToDate') }}</h4>
          <p>{{ t('update.lastChecked') }}: {{ formatDate(lastChecked) }}</p>
        </div>
      </div>

      <div v-if="updateCheck?.hasUpdate && !installing" class="update-available">
        <div class="update-header">
          <Download :size="24" class="update-icon" />
          <div class="update-details">
            <h4>{{ t('update.updateAvailable') }}</h4>
            <div class="version-comparison">
              <span class="current">v{{ updateCheck.currentVersion }}</span>
              <ArrowRight :size="16" />
              <span class="new">{{ updateCheck.latestVersion }}</span>
            </div>
            <div class="release-date">{{ t('update.releaseDate') }}: {{ formatDate(updateCheck.releaseDate) }}</div>
          </div>
        </div>

        <div v-if="updateCheck.releaseNotes" class="release-notes">
          <h5>{{ t('update.releaseNotes') }}</h5>
          <div class="notes-content" v-html="formatReleaseNotes(updateCheck.releaseNotes)"></div>
        </div>

        <div v-if="updateCheck.assets.length > 0" class="assets-section">
          <h5>{{ t('update.availableInstallers') }}</h5>
          <div class="assets-list">
            <div
              v-for="asset in updateCheck.assets"
              :key="asset.name"
              class="asset-item"
              :class="{ selected: selectedAsset?.name === asset.name }"
              @click="selectAsset(asset)"
            >
              <div class="asset-info">
                <div class="asset-name">{{ asset.name }}</div>
                <div class="asset-details">
                  <span class="asset-type">{{ asset.type.toUpperCase() }}</span>
                  <span class="asset-size">{{ formatBytes(asset.size) }}</span>
                </div>
              </div>
              <div class="asset-platform">{{ asset.platform }}</div>
            </div>
          </div>
        </div>

        <div v-if="downloading" class="progress-container">
          <div class="progress-info">
            <span class="progress-text">{{ t('update.downloading') }}...</span>
            <span class="progress-percentage">{{ Math.round(downloadProgress?.percentage || 0) }}%</span>
          </div>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${downloadProgress?.percentage || 0}%` }"></div>
          </div>
          <div class="progress-details">
            <span class="speed">{{ formatSpeed(downloadProgress?.speed || 0) }}</span>
            <span class="size-info">
              {{ formatBytes(downloadProgress?.downloaded || 0) }} / {{ formatBytes(downloadProgress?.total || 0) }}
            </span>
          </div>
        </div>

        <div v-if="!downloading" class="update-actions">
          <button
            :disabled="!selectedAsset || checking || downloading || installing"
            class="install-btn"
            @click="downloadAndInstall"
          >
            <Download :size="16" />
            {{ t('update.downloadAndInstall') }}
          </button>
        </div>
      </div>
      <div v-if="backgroundUpdateAvailable" class="background-update-notification">
        <div class="notification-content">
          <Info :size="20" class="notification-icon" />
          <div class="notification-text">
            <span>{{ t('update.backgroundUpdateAvailable') }}</span>
            <button class="show-update-btn" @click="showBackgroundUpdate">
              {{ t('update.showUpdate') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle, ArrowRight, CheckCircle, Download, Info, RefreshCw } from 'lucide-vue-next'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import useMessage from '@/hooks/useMessage'
import { formatBytes } from '@/utils/formatters'

import { TauriAPI } from '../../api/tauri'
import { useTranslation } from '../../composables/useI18n'
import { useAppStore } from '../../stores/app'
import CustomButton from '../common/CustomButton.vue'
import CustomNavCard from '../common/CustomNavCard.vue'
import CustomSwitch from '../common/CustomSwitch.vue'
import SettingCard from '../common/SettingCard.vue'
import SettingSection from '../common/SettingSection.vue'

const { t } = useTranslation()
const appStore = useAppStore()
const message = useMessage()

const currentVersion = ref('')
const updateCheck = ref<UpdateCheck | null>(null)
const backgroundUpdateCheck = ref<UpdateCheck | null>(null)
const checking = ref(false)
const downloading = ref(false)
const installing = ref(false)
const downloadProgress = ref<DownloadProgress | null>(null)
const lastChecked = ref<string | null>(null)
const error = ref<string | null>(null)
const autoCheckEnabled = ref(true)
const settingsLoading = ref(false)
const selectedAsset = ref<UpdateAsset | null>(null)

const backgroundUpdateAvailable = computed(() => backgroundUpdateCheck.value && !updateCheck.value?.hasUpdate)

let backgroundUpdateUnlisten: (() => void) | null = null
let downloadProgressUnlisten: (() => void) | null = null
let installStartedUnlisten: (() => void) | null = null
let installErrorUnlisten: (() => void) | null = null
let appQuitEventUnsubscriber: (() => void) | null = null

const checkForUpdates = async () => {
  if (checking.value || downloading.value || installing.value) return

  try {
    checking.value = true
    error.value = null

    const result = await TauriAPI.updater.check()
    updateCheck.value = result

    if (result.hasUpdate && result.assets.length > 0) {
      selectedAsset.value = result.assets[0]
    }

    lastChecked.value = new Date().toISOString()

    if (!result.hasUpdate) {
      message.info(t('update.noUpdatesFound'))
    }
  } catch (err: any) {
    console.error('Failed to check for updates:', err)
    error.value = t('update.checkError') + String(err ? `: ${err}` : '')
  } finally {
    checking.value = false
  }
}

const selectAsset = (asset: UpdateAsset) => {
  selectedAsset.value = asset
}

const downloadAndInstall = async () => {
  if (!selectedAsset.value || downloading.value || installing.value) return

  try {
    downloading.value = true
    message.info(t('update.startingDownload'))

    const filePath = await TauriAPI.updater.download(selectedAsset.value.url, selectedAsset.value.name)

    downloading.value = false
    installing.value = true
    message.info(t('update.installingUpdate'))

    await TauriAPI.updater.installAndRestart(filePath)
  } catch (err: any) {
    console.error('Failed to download/install update:', err)
    downloading.value = false
    installing.value = false
    error.value = err.message || t('update.installError')
    message.error(t('update.installError'))
  }
}

const toggleAutoCheck = async () => {
  if (settingsLoading.value) return

  try {
    settingsLoading.value = true
    await TauriAPI.updater.setAutoCheck(autoCheckEnabled.value)
  } catch (err: any) {
    console.error('Failed to update auto-check setting:', err)
    autoCheckEnabled.value = !autoCheckEnabled.value
  } finally {
    settingsLoading.value = false
  }
}

const showBackgroundUpdate = () => {
  if (backgroundUpdateCheck.value) {
    updateCheck.value = backgroundUpdateCheck.value
    backgroundUpdateCheck.value = null
    if (updateCheck.value.assets.length > 0) {
      selectedAsset.value = updateCheck.value.assets[0]
    }
  }
}

const clearError = () => {
  error.value = null
}

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString()
  } catch {
    return dateString
  }
}

const formatReleaseNotes = (notes: string) => {
  return notes
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/\n/g, '<br>')
}

const formatSpeed = (bytesPerSecond: number) => {
  if (bytesPerSecond === 0) return '0 B/s'
  const k = 1024
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k))
  return parseFloat((bytesPerSecond / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

onMounted(async () => {
  try {
    if (appStore.updateAvailable && appStore.updateCheck) {
      updateCheck.value = appStore.updateCheck
      if (appStore.updateCheck.assets.length > 0) {
        selectedAsset.value = appStore.updateCheck.assets[0]
      }
    }
    appStore.clearUpdateStatus()
    currentVersion.value = await TauriAPI.updater.currentVersion()
    autoCheckEnabled.value = await TauriAPI.updater.isAutoCheckEnabled()
    try {
      backgroundUpdateUnlisten = await TauriAPI.updater.onBackgroundUpdate(updateInfo => {
        console.log('Background update available:', updateInfo)
        backgroundUpdateCheck.value = updateInfo
      })
    } catch (err) {
      console.warn('Background update listener not available:', err)
      backgroundUpdateUnlisten = null
    }

    try {
      downloadProgressUnlisten = await TauriAPI.updater.onDownloadProgress(progress => {
        downloadProgress.value = progress
      })
    } catch (err) {
      console.warn('Download progress listener not available:', err)
      downloadProgressUnlisten = null
    }

    try {
      installStartedUnlisten = await TauriAPI.updater.onInstallStarted(() => {
        installing.value = true
        message.info(t('update.installingUpdate'))
      })
    } catch (err) {
      console.warn('Install started listener not available:', err)
      installStartedUnlisten = null
    }

    try {
      installErrorUnlisten = await TauriAPI.updater.onInstallError(errorMsg => {
        installing.value = false
        error.value = errorMsg
        message.error(t('update.installError'))
      })
    } catch (err) {
      console.warn('Install error listener not available:', err)
      installErrorUnlisten = null
    }

    try {
      appQuitEventUnsubscriber = await TauriAPI.updater.onAppQuit(() => {
        message.success(t('update.quitApp'))
      })
    } catch (err) {
      console.warn('App restarting listener not available:', err)
      appQuitEventUnsubscriber = null
    }
    if (autoCheckEnabled.value) {
      await checkForUpdates()
    }
  } catch (err) {
    console.error('Failed to initialize update manager:', err)
  }
})

onUnmounted(() => {
  try {
    backgroundUpdateUnlisten?.()
  } catch (err) {
    console.warn('Error unregistering background update listener:', err)
  }

  try {
    downloadProgressUnlisten?.()
  } catch (err) {
    console.warn('Error unregistering download progress listener:', err)
  }

  try {
    installStartedUnlisten?.()
  } catch (err) {
    console.warn('Error unregistering install started listener:', err)
  }

  try {
    installErrorUnlisten?.()
  } catch (err) {
    console.warn('Error unregistering install error listener:', err)
  }

  try {
    appQuitEventUnsubscriber?.()
  } catch (err) {
    console.warn('Error unregistering app restarting listener:', err)
  }
})
</script>

<style scoped>
.update-manager-card {
  max-width: 700px;
}

.update-manager-card.standalone {
  max-width: 100%;
  width: 100%;
  box-shadow: none;
  border: 1px solid var(--color-border);
}

.update-content {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.version-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background: var(--color-surface);
  border-radius: 8px;
}

.current-version h4 {
  margin: 0 0 0.5rem 0;
  font-size: 0.9rem;
  color: var(--color-text-secondary);
}

.version-tag {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-primary);
}

.check-update-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.check-update-btn:hover:not(:disabled) {
  background: rgb(39, 221, 145);
}

.check-update-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.settings-row {
  padding: 1rem;
  background: var(--color-surface);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.auto-check-setting {
  display: flex;
  align-items: center;
}

.checkbox-container {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  user-select: none;
}

.checkbox-container input[type='checkbox'] {
  width: 18px;
  height: 18px;
  accent-color: var(--color-primary);
}

.checkbox-container .label-text {
  font-size: 0.9rem;
  color: var(--color-text);
}

.error-state {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  background: rgba(255, 59, 48, 0.1);
  border: 1px solid rgba(255, 59, 48, 0.2);
  border-radius: 6px;
  color: var(--color-danger);
}

:root.dark .error-state,
:root.auto.dark .error-state {
  background: rgba(255, 59, 48, 0.05);
  border-color: rgba(255, 59, 48, 0.2);
}

.error-content {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
}

.clear-error-btn {
  background: none;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  padding: 0.25rem;
  color: var(--color-danger);
  opacity: 0.7;
}

.clear-error-btn:hover {
  opacity: 1;
}

.no-updates {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.5rem;
  background: var(--color-surface);
  border-radius: 8px;
}

.check-icon {
  color: var(--color-success);
}

.no-updates-text h4 {
  margin: 0 0 0.25rem 0;
}

.no-updates-text p {
  margin: 0;
  font-size: 0.9rem;
  color: var(--color-text-secondary);
}

.update-available {
  border: 2px solid var(--color-success);
  border-radius: 12px;
  padding: 1.5rem;
  background: rgba(52, 199, 89, 0.1);
}

:root.dark .update-available,
:root.auto.dark .update-available {
  background: rgba(52, 199, 89, 0.05);
  border-color: var(--color-success);
}

.update-header {
  display: flex;
  gap: 1rem;
  margin-bottom: 1rem;
}

.update-icon {
  color: var(--color-success);
  flex-shrink: 0;
}

.update-details h4 {
  margin: 0 0 0.5rem 0;
  color: var(--color-success);
}

.version-comparison {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.version-comparison .current {
  color: var(--color-text-secondary);
}

.version-comparison .new {
  color: var(--color-success);
  font-weight: 600;
}

.release-date {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
}

.release-notes {
  margin: 1rem 0;
}

.release-notes h5 {
  margin: 0 0 0.5rem 0;
  font-size: 0.9rem;
}

.notes-content {
  max-height: 200px;
  overflow-y: auto;
  padding: 0.75rem;
  background: rgba(255, 255, 255, 0.7);
  border-radius: 6px;
  font-size: 0.85rem;
  line-height: 1.4;
}

:root.dark .notes-content,
:root.auto.dark .notes-content {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-primary);
}

.assets-section {
  margin: 1rem 0;
}

.assets-section h5 {
  margin: 0 0 0.75rem 0;
  font-size: 0.9rem;
  color: var(--color-success);
}

.assets-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.asset-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background: rgba(255, 255, 255, 0.8);
  border: 2px solid transparent;
  border-radius: 6px;
  cursor: pointer;
}

.asset-item:hover {
  background: rgba(255, 255, 255, 0.9);
}

.asset-item.selected {
  border-color: var(--color-success);
  background: rgba(255, 255, 255, 0.95);
}

:root.dark .asset-item,
:root.auto.dark .asset-item {
  background: rgba(255, 255, 255, 0.05);
}

:root.dark .asset-item:hover,
:root.auto.dark .asset-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

:root.dark .asset-item.selected,
:root.auto.dark .asset-item.selected {
  background: rgba(255, 255, 255, 0.1);
  border-color: var(--color-success);
}

.asset-info {
  flex: 1;
}

.asset-name {
  font-weight: 500;
  margin-bottom: 0.25rem;
}

.asset-details {
  display: flex;
  gap: 1rem;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.asset-type {
  font-weight: 600;
  padding: 0.125rem 0.5rem;
  background: var(--color-primary);
  color: white;
  border-radius: 3px;
  text-transform: uppercase;
  font-size: 0.7rem;
}

.asset-platform {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.progress-container {
  margin: 1rem 0;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.progress-text {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--color-success);
}

.progress-percentage {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-success);
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: rgba(255, 255, 255, 0.7);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 0.5rem;
}

:root.dark .progress-bar,
:root.auto.dark .progress-bar {
  background: rgba(255, 255, 255, 0.1);
}

.progress-fill {
  height: 100%;
  background: var(--color-success);
}

.progress-details {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.update-actions {
  display: flex;
  gap: 0.75rem;
}

.install-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  background: var(--color-success);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.install-btn:hover:not(:disabled) {
  background: rgba(52, 199, 89, 0.8);
}

.install-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.status-message {
  padding: 0.75rem;
  border-radius: 6px;
  margin-top: 1rem;
}

.status-message.info {
  background: rgba(0, 122, 255, 0.1);
  border: 1px solid rgba(0, 122, 255, 0.2);
  color: var(--color-accent);
}

.status-message.success {
  background: rgba(52, 199, 89, 0.1);
  border: 1px solid rgba(52, 199, 89, 0.2);
  color: var(--color-success);
}

.status-message.error {
  background: rgba(255, 59, 48, 0.1);
  border: 1px solid rgba(255, 59, 48, 0.2);
  color: var(--color-danger);
}

:root.dark .status-message.info,
:root.auto.dark .status-message.info {
  background: rgba(10, 132, 255, 0.05);
  border-color: rgba(10, 132, 255, 0.2);
  color: var(--color-accent);
}

:root.dark .status-message.success,
:root.auto.dark .status-message.success {
  background: rgba(52, 199, 89, 0.05);
  border-color: rgba(52, 199, 89, 0.2);
  color: var(--color-success);
}

:root.dark .status-message.error,
:root.auto.dark .status-message.error {
  background: rgba(255, 59, 48, 0.05);
  border-color: rgba(255, 59, 48, 0.2);
  color: var(--color-danger);
}

.status-content {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.background-update-notification {
  padding: 1rem;
  background: rgba(0, 122, 255, 0.1);
  border: 1px solid rgba(0, 122, 255, 0.2);
  border-radius: 8px;
  color: var(--color-accent);
}

:root.dark .background-update-notification,
:root.auto.dark .background-update-notification {
  background: rgba(10, 132, 255, 0.05);
  border-color: rgba(10, 132, 255, 0.2);
}

.notification-content {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.notification-icon {
  color: var(--color-accent);
  flex-shrink: 0;
}

.notification-text {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex: 1;
}

.show-update-btn {
  padding: 0.25rem 0.75rem;
  background: var(--color-accent);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
}

.show-update-btn:hover {
  background: var(--color-accent-hover);
}

/* Dark mode specific improvements */
:root.dark .asset-type,
:root.auto.dark .asset-type {
  background: var(--color-primary);
  color: white;
}

:root.dark .asset-details,
:root.auto.dark .asset-details {
  color: var(--color-text-secondary);
}

:root.dark .asset-platform,
:root.auto.dark .asset-platform {
  color: var(--color-text-secondary);
}

:root.dark .progress-details,
:root.auto.dark .progress-details {
  color: var(--color-text-secondary);
}

:root.dark .update-icon,
:root.auto.dark .update-icon {
  color: var(--color-success);
}

:root.dark .install-btn,
:root.auto.dark .install-btn {
  background: var(--color-success);
  color: white;
}

:root.dark .install-btn:hover:not(:disabled),
:root.auto.dark .install-btn:hover:not(:disabled) {
  background: rgba(52, 199, 89, 0.8);
}
</style>
