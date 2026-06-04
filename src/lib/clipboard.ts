import { writeText as tauriWriteText, readText as tauriReadText } from '@tauri-apps/plugin-clipboard-manager'

export function writeText(text: string): Promise<void> {
  return tauriWriteText(text)
}

export function readText(): Promise<string> {
  return tauriReadText()
}
