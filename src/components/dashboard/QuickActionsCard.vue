<template>
  <div class="flex flex-col gap-4 w-full justify-center p-4">
    <div class="flex gap-2 justify-start items-center">
      <Settings class="text-accent" />
      <h4 class="font-semibold text-main">{{ t('dashboard.quickActions.title') }}</h4>
    </div>
    <div class="flex flex-row w-full gap-4">
      <div class="flex flex-col flex-1 gap-2 border p-2 rounded-md border-border-secondary shadow-sm">
        <div class="flex flex-wrap gap-2 items-center">
          <h4 class="text-main font-semibold text-sm">{{ t('dashboard.quickActions.openlistService') }}</h4>
          <div v-if="isCoreLoading" class="flex items-center">
            <div class="border-3 border-border w-4 h-4 rounded-full border-t-3 border-t-accent animate-spin"></div>
          </div>
        </div>
        <div class="flex flex-wrap gap-2 items-center w-full">
          <div>
            <CustomButton v-if="isCoreLoading" :icon="Loader" text="" type="secondary" disabled />
            <CustomButton
              v-else-if="isCoreRunning"
              type="custom"
              class="bg-danger/80! hover:bg-danger!"
              text-class="text-white"
              icon-class="text-white"
              :icon="Square"
              :text="t('dashboard.quickActions.stopOpenListCore')"
              @click="toggleCore"
            />
            <CustomButton
              v-else
              type="primary"
              :icon="Play"
              :text="t('dashboard.quickActions.startOpenListCore')"
              @click="toggleCore"
            />
          </div>

          <CustomButton
            type="secondary"
            :disabled="!isCoreRunning || isCoreLoading"
            :icon="isCoreLoading ? Loader : RotateCcw"
            :text="t('dashboard.quickActions.restart')"
            @click="restartCore"
          />

          <CustomButton
            type="secondary"
            :disabled="!isCoreRunning || isCoreLoading"
            :icon="ExternalLink"
            :text="t('dashboard.quickActions.openWeb')"
            @click="openWebUI"
          />

          <CustomButton
            type="secondary"
            :icon="Key"
            text=""
            :title="t('dashboard.quickActions.copyAdminPassword')"
            @click="copyAdminPassword"
          />
          <CustomButton
            type="secondary"
            :icon="RotateCcw"
            text=""
            :title="t('dashboard.quickActions.resetAdminPassword')"
            @click="resetAdminPassword"
          />
          <CustomButton
            v-if="isWindows"
            type="custom"
            :class="{
              'bg-success/80 hover:bg-success! text-white': !firewallEnabled,
              'bg-danger/80 hover:bg-danger! text-white': firewallEnabled,
            }"
            text-class="text-white"
            :disabled="firewallLoading"
            :icon="firewallLoading ? Loader : Shield"
            :text="
              firewallEnabled
                ? t('dashboard.quickActions.firewall.disable')
                : t('dashboard.quickActions.firewall.enable')
            "
            @click="toggleFirewallRule"
          />
        </div>
      </div>
      <div class="flex flex-wrap flex-col flex-1 gap-2 border p-2 rounded-md border-border-secondary shadow-sm">
        <div class="flex flex-wrap gap-2 items-center">
          <h4 class="text-main font-semibold text-sm">{{ t('dashboard.quickActions.rclone') }}</h4>
        </div>
        <div class="flex flex-wrap gap-2 items-center w-full">
          <CustomButton
            type="secondary"
            :icon="Settings"
            class="flex-1!"
            :text="t('dashboard.quickActions.configRclone')"
            @click="openRcloneConfig"
          />
          <CustomButton
            type="secondary"
            :icon="HardDrive"
            class="flex-1!"
            :text="t('dashboard.quickActions.manageMounts')"
            @click="viewMounts"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ExternalLink, HardDrive, Key, Loader, Play, RotateCcw, Settings, Shield, Square } from 'lucide-vue-next'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { TauriAPI } from '@/api/tauri'
import useMessage from '@/hooks/useMessage'
import { createNewWindow } from '@/utils/common'

import { useTranslation } from '../../composables/useI18n'
import { useAppStore } from '../../stores/app'
import CustomButton from '../common/CustomButton.vue'

const { t } = useTranslation()
const router = useRouter()
const message = useMessage()
const appStore = useAppStore()

const isCoreRunning = computed(() => appStore.isCoreRunning)
const isCoreLoading = computed(() => appStore.loading)
const statusCheckInterval: number | null = null

const firewallEnabled = ref(false)
const firewallLoading = ref(false)
const isWindows = computed(() => {
  return typeof OS_PLATFORM !== 'undefined' && OS_PLATFORM === 'win32'
})

const toggleCore = async () => {
  if (isCoreRunning.value) {
    await appStore.stopOpenListCore()
  } else {
    await appStore.startOpenListCore()
  }
}

const restartCore = async () => {
  await appStore.restartOpenListCore()
}

const openWebUI = () => {
  if (appStore.openListCoreUrl) {
    openLink(appStore.openListCoreUrl)
  }
}

const openRcloneConfig = () => {
  router.push({ name: 'Settings', query: { tab: 'rclone' } })
}

const viewMounts = () => {
  router.push({ name: 'Mount' })
}

const copyAdminPassword = async () => {
  try {
    const password = await appStore.getAdminPassword()
    if (password) {
      await navigator.clipboard.writeText(password)
      message.success('Admin password copied: ' + password)
    } else {
      message.error('No admin password found.')
    }
  } catch (error) {
    console.error('Failed to get admin password:', error)
    message.error('Failed to get admin password. Please check the logs.')
  }
}

const resetAdminPassword = async () => {
  try {
    const newPassword = await appStore.resetAdminPassword()
    if (newPassword) {
      await navigator.clipboard.writeText(newPassword)
      message.success('Admin password reset and copied: ' + newPassword)
    } else {
      message.error('Failed to reset admin password. Please check the logs.')
    }
  } catch (error) {
    console.error('Failed to reset admin password:', error)
    message.error('Failed to reset admin password. Please check the logs.')
  }
}

const checkFirewallStatus = async () => {
  if (!isWindows.value) return

  try {
    firewallEnabled.value = await TauriAPI.firewall.check()
  } catch (error) {
    console.error('Failed to check firewall status:', error)
  }
}

const toggleFirewallRule = async () => {
  if (!isWindows.value) return

  try {
    firewallLoading.value = true

    if (firewallEnabled.value) {
      await TauriAPI.firewall.remove()
      firewallEnabled.value = false
      message.success(t('dashboard.quickActions.firewall.removed'))
    } else {
      await TauriAPI.firewall.add()
      firewallEnabled.value = true
      message.success(t('dashboard.quickActions.firewall.added'))
    }
  } catch (error: any) {
    console.error('Failed to toggle firewall rule:', error)
    const msg = firewallEnabled.value
      ? t('dashboard.quickActions.firewall.failedToRemove')
      : t('dashboard.quickActions.firewall.failedToAdd')
    message.error(msg + ': ' + (error.message || error))
  } finally {
    firewallLoading.value = false
  }
}

const isMacOs = computed(() => {
  return typeof OS_PLATFORM !== 'undefined' && OS_PLATFORM === 'darwin'
})

const openLink = async (url: string) => {
  try {
    if (appStore.settings.app.open_links_in_browser || isMacOs.value) {
      await TauriAPI.files.urlInBrowser(url)
      return
    }
  } catch (error) {
    console.error('Failed to open link:', error)
  }
  createNewWindow(url, `webview-${Date.now()}`, 'External Link')
}

onMounted(async () => {
  await checkFirewallStatus()
})

onUnmounted(() => {
  if (statusCheckInterval) {
    clearInterval(statusCheckInterval)
  }
})
</script>
