import { onOpenUrl as tauriOnOpenUrl, getCurrent as tauriGetCurrent } from '@tauri-apps/plugin-deep-link'

export function onOpenUrl(callback: (urls: string[]) => void): Promise<() => void> {
  return tauriOnOpenUrl(callback)
}

export function getCurrent(): Promise<string[] | null> {
  return tauriGetCurrent()
}
