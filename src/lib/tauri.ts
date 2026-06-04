import { invoke as tauriInvoke } from '@tauri-apps/api/core'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const win = typeof window !== 'undefined' ? (window as any) : null
  const mock = win?.['__TAURI__'] as any
  const mockInvoke = mock?.core?.invoke as ((c: string, a?: Record<string, unknown>) => Promise<T>) | undefined
  if (mockInvoke) {
    return mockInvoke(cmd, args)
  }

  return tauriInvoke(cmd, args)
}