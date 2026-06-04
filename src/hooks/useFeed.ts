import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import type { FeedHighlight } from '../types'

export interface UseFeedReturn {
  feed: FeedHighlight[]
  error: string | null
  loadFeed: () => Promise<void>
}

export function useFeed(): UseFeedReturn {
  const [feed, setFeed] = useState<FeedHighlight[]>([])
  const [error, setError] = useState<string | null>(null)

  const loadFeed = useCallback(async () => {
    setError(null)
    try {
      const result: FeedHighlight[] = await invoke('get_subscriber_feed', {
        limit: 50,
        offset: 0,
      })
      setFeed(result)
    } catch (err) {
      setError(String(err))
    }
  }, [])

  return { feed, error, loadFeed }
}
