import { ref } from 'vue'

export interface MenuItem {
  label?: string
  shortcut?: string
  action?: () => void
  enabled?: boolean
  type?: 'separator'
}

export interface MenuSection {
  label: string
  items: MenuItem[]
}

export function useMenu() {
  const showMenuDropdown = ref<string | false>(false)
  const showUserMenu = ref(false)

  const closeAllMenus = () => {
    showMenuDropdown.value = false
    showUserMenu.value = false
  }

  const toggleMenu = (menuName: string) => {
    if (showMenuDropdown.value === menuName) {
      showMenuDropdown.value = false
    } else {
      showMenuDropdown.value = menuName
      showUserMenu.value = false
    }
  }

  return {
    showMenuDropdown,
    showUserMenu,
    closeAllMenus,
    toggleMenu
  }
}
