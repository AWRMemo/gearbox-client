import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '../lib/tauri'
import { useToast } from '../hooks'
import type { SyncStatus, Conflict, ConflictResolution } from '../types'

export interface UseSyncReturn {
  syncStatus: SyncStatus | null
  conflicts: Conflict[]
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
  syncNow: () => Promise<void>
  resolveConflict: (id: string, resolution: ConflictResolution) => Promise<void>
}

export function useSync(): UseSyncReturn {
  const { toast } = useToast()
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null)
  const [conflicts, setConflicts] = useState<Conflict[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const initialized = useRef(false)

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      const status: SyncStatus = await invoke('get_sync_status')
      setSyncStatus(status)
      const list: Conflict[] = await invoke('get_conflicts')
      setConflicts(list)
      setError(null)
    } catch (err) {
      const msg = `Failed to load sync status: ${String(err)}`
      setError(msg)
    } finally {
      setLoading(false)
    }
  }, [])

  const syncNow = useCallback(async () => {
    setLoading(true)
    try {
      const report = await invoke<{ pushed: number; pulled: number; conflicts: number }>('sync_now')
      toast({ message: `Sync complete: +${report.pulled} pulled, ${report.pushed} pushed`, type: 'success' })
      await refresh()
    } catch (err) {
      const msg = `Sync failed: ${String(err)}`
      setError(msg)
        toast({ message: msg, type: 'error' })
    } finally {
      setLoading(false)
    }
  }, [refresh, toast])

  const resolveConflict = useCallback(
    async (id: string, resolution: ConflictResolution) => {
      setLoading(true)
      try {
        await invoke('resolve_conflict', { id, resolution })
        toast({ message: 'Conflict resolved', type: 'success' })
        await refresh()
      } catch (err) {
        const msg = `Resolution failed: ${String(err)}`
        setError(msg)
      toast({ message: msg, type: 'error' })
      } finally {
        setLoading(false)
      }
    },
    [refresh, toast]
  )

  useEffect(() => {
    if (!initialized.current) {
      initialized.current = true
      queueMicrotask(() => refresh())
    }
  }, [refresh])

  return { syncStatus, conflicts, loading, error, refresh, syncNow, resolveConflict }
}
