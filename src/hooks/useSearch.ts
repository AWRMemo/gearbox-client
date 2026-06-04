import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import { useToast } from './useToast'
import type { SearchResult } from '../types'

export interface SearchFilter {
  dateFrom?: string
  dateTo?: string
  tags?: string[]
  sourceDomain?: string
}

export interface UseSearchReturn {
  query: string
  setQuery: (q: string) => void
  results: SearchResult[]
  isSearching: boolean
  error: string | null
  selectedIndex: number
  semantic: boolean
  setSemantic: (v: boolean) => void
  filters: SearchFilter
  setFilters: (f: SearchFilter) => void
  handleSearch: () => Promise<void>
  handleKeyDown: (e: React.KeyboardEvent) => void
  moveSelection: (dir: 'up' | 'down') => void
}

export function useSearch(): UseSearchReturn {
  const { toast } = useToast()
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [isSearching, setIsSearching] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const [semantic, setSemantic] = useState(false)
  const [filters, setFilters] = useState<SearchFilter>({})

  const handleSearch = useCallback(async () => {
    const q = query.trim()
    if (!q) return
    setIsSearching(true)
    setError(null)
    setResults([])
    setSelectedIndex(-1)
    try {
      const res: SearchResult[] = await invoke('search', {
        query: q,
        limit: 20,
        dateFrom: filters.dateFrom || null,
        dateTo: filters.dateTo || null,
        sourceDomain: filters.sourceDomain || null,
      })
      setResults(res)
    } catch (err) {
      const msg = String(err)
      setError(msg)
      toast({ message: msg, type: 'error' })
    } finally {
      setIsSearching(false)
    }
  }, [query, toast])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && selectedIndex >= 0) {
        return
      }
      if (e.key === 'Enter') {
        handleSearch()
        return
      }
      if (e.key === 'Escape') {
        setQuery('')
        setResults([])
        setSelectedIndex(-1)
        return
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1))
        return
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault()
        setSelectedIndex((prev) => Math.max(prev - 1, -1))
        return
      }
    },
    [handleSearch, results.length, selectedIndex]
  )

  const moveSelection = useCallback((dir: 'up' | 'down') => {
    setSelectedIndex((prev) => {
      if (dir === 'down') return Math.min(prev + 1, results.length - 1)
      return Math.max(prev - 1, -1)
    })
  }, [results.length])

  return {
    query,
    setQuery,
    results,
    isSearching,
    error,
    selectedIndex,
    semantic,
    setSemantic,
    filters,
    setFilters,
    handleSearch,
    handleKeyDown,
    moveSelection,
  }
}
