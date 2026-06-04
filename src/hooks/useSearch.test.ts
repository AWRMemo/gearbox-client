import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { useSearch } from './useSearch'

vi.mock('../lib/tauri', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '../lib/tauri'
const mockInvoke = invoke as ReturnType<typeof vi.fn>

describe('useSearch', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('has empty initial state', () => {
    const { result } = renderHook(() => useSearch())
    expect(result.current.query).toBe('')
    expect(result.current.results).toEqual([])
    expect(result.current.isSearching).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('updates query via setter', () => {
    const { result } = renderHook(() => useSearch())
    act(() => result.current.setQuery('fox'))
    expect(result.current.query).toBe('fox')
  })

  it('returns results on successful search', async () => {
    const mockResults = [
      { id: '1', summary: 'A fox', tags: ['fox'], text: 'The quick brown fox.', score: 0.9 },
    ]
    mockInvoke.mockResolvedValue(mockResults)

    const { result } = renderHook(() => useSearch())
    act(() => result.current.setQuery('fox'))
    await act(async () => await result.current.handleSearch())

    expect(result.current.results.length).toBe(1)
    expect(result.current.results[0].id).toBe('1')
    expect(result.current.isSearching).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('sets error on failed search', async () => {
    mockInvoke.mockRejectedValue(new Error('DB locked'))

    const { result } = renderHook(() => useSearch())
    act(() => result.current.setQuery('fox'))
    await act(async () => await result.current.handleSearch())

    await waitFor(() => expect(result.current.error).not.toBeNull())
    expect(result.current.error).toContain('DB locked')
    expect(result.current.isSearching).toBe(false)
  })

  it('does nothing when query is empty', async () => {
    const { result } = renderHook(() => useSearch())
    await act(async () => await result.current.handleSearch())
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('triggers search on Enter key', () => {
    const { result } = renderHook(() => useSearch())
    act(() => result.current.handleKeyDown({ key: 'Enter' } as React.KeyboardEvent))
    expect(result.current.query).toBe('')
  })
})
