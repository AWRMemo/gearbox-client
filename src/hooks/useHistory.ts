import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import { useToast } from './useToast'
import type { ListedHighlight } from '../types'

export interface UseHistoryReturn {
  entries: ListedHighlight[]
  highlights: ListedHighlight[] // backward-compatible alias
  isLoading: boolean
  hasMore: boolean
  loadMore: () => Promise<void>
  refresh: () => Promise<void>
  deleteHighlight: (id: string) => Promise<void>
}

const DEFAULT_PAGE_SIZE = 20

export function useHistory(limit?: number, offset?: number): UseHistoryReturn {
  const { toast } = useToast()
  const pageSize = limit ?? DEFAULT_PAGE_SIZE
  const initialOffset = offset ?? 0
  const [entries, setEntries] = useState<ListedHighlight[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [hasMore, setHasMore] = useState(true)

  const loadPage = useCallback(async (pageOffset: number) => {
    setIsLoading(true)
    try {
      const page: ListedHighlight[] = await invoke('get_history_paginated', {
        limit: pageSize,
        offset: pageOffset,
      })
      setHasMore(page.length === pageSize)
      if (pageOffset === initialOffset) {
        setEntries(page)
      } else {
        setEntries(prev => [...prev, ...page])
      }
    } catch (err) {
      toast({ message: `Failed to load history: ${String(err)}`, type: 'error' })
    } finally {
      setIsLoading(false)
    }
  }, [pageSize, initialOffset, toast])

  const loadMore = useCallback(async () => {
    if (isLoading || !hasMore) return
    await loadPage(entries.length)
  }, [isLoading, hasMore, entries.length, loadPage])

  const refresh = useCallback(async () => {
    setHasMore(true)
    await loadPage(initialOffset)
  }, [loadPage, initialOffset])

  const deleteHighlight = useCallback(async (id: string) => {
    try {
      await invoke('delete_highlight', { id })
      await refresh()
      toast({ message: 'Highlight deleted', type: 'info' })
    } catch (err) {
      toast({ message: `Failed to delete highlight: ${String(err)}`, type: 'error' })
    }
  }, [refresh, toast])

  return {
    entries,
    highlights: entries,
    isLoading,
    hasMore,
    loadMore,
    refresh,
    deleteHighlight,
  }
}
