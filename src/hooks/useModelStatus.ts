import { useState, useCallback, useEffect } from 'react'
import { invoke } from '../lib/tauri'
import type { ModelStatus } from '../types'

export interface UseModelStatusReturn {
  status: ModelStatus | null
  load: () => Promise<void>
}

export function useModelStatus(): UseModelStatusReturn {
  const [status, setStatus] = useState<ModelStatus | null>(null)

  const load = useCallback(async () => {
    try {
      const s: ModelStatus = await invoke('get_model_status')
      setStatus(s)
    } catch {
      setStatus({ loaded: false, model_name: null, embedding_available: false, download_progress: null, download_state: 'idle' })
    }
  }, [])

  useEffect(() => {
    let active = true
    async function tick() {
      try {
        const s: ModelStatus = await invoke('get_model_status')
        if (active) setStatus(s)
      } catch {
        if (active) setStatus({ loaded: false, model_name: null, embedding_available: false, download_progress: null, download_state: 'idle' })
      }
    }
    tick()
    const id = setInterval(tick, 5000)
    return () => {
      active = false
      clearInterval(id)
    }
  }, [])

  return { status, load }
}
