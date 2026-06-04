import { Channel as TauriChannel } from '@tauri-apps/api/core'

export function createChannel<T>(): TauriChannel<T> {
  return new TauriChannel<T>()
}

export type ChannelType<T> = TauriChannel<T>
