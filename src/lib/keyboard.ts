import { register } from '@tauri-apps/plugin-global-shortcut'

let searchInputRef: HTMLInputElement | null = null

export function setSearchInputRef(el: HTMLInputElement | null) {
  searchInputRef = el
}

export async function registerKeyboardShortcuts(onCapture: () => void) {
  try {
    await register('CommandOrControl+Shift+C', (event) => {
      if (event.state === 'Pressed') {
        onCapture()
      }
    })
    await register('CommandOrControl+K', (event) => {
      if (event.state === 'Pressed') {
        searchInputRef?.focus()
      }
    })
  } catch (e) {
    console.warn('Global shortcut registration failed:', e)
  }
}
