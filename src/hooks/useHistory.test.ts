import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { useHistory } from './useHistory'

vi.mock('../lib/tauri', () => ({ invoke: vi.fn() }))
vi.mock('./useToast', () => ({ useToast: () => ({ toast: vi.fn() }) }))

import { invoke } from '../lib/tauri'
const mockInvoke = invoke as ReturnType<typeof vi.fn>

describe('useHistory', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('has empty initial state', () => {
    const { result } = renderHook(() => useHistory())
    expect(result.current.entries).toEqual([])
    expect(result.current.highlights).toEqual([])
    expect(result.current.isLoading).toBe(false)
    expect(result.current.hasMore).toBe(true)
  })

  it('loads first page on refresh', async () => {
    const page = Array.from({ length: 20 }, (_, i) => ({
      id: `h${i}`,
      text: `text ${i}`,
      source_url: null,
      source_title: null,
      source_author: null,
      summary: `summary ${i}`,
      tags: ['a'],
      created_at: '2024-01-01',
    }))
    mockInvoke.mockResolvedValue(page)

    const { result } = renderHook(() => useHistory())
    await act(async () => await result.current.refresh())

    await waitFor(() => expect(result.current.entries.length).toBe(20))
    expect(result.current.hasMore).toBe(true)
    expect(mockInvoke).toHaveBeenCalledWith('get_history_paginated', { limit: 20, offset: 0 })
  })

  it('paginates and sets hasMore false on partial page', async () => {
    const page1 = Array.from({ length: 20 }, (_, i) => ({
      id: `h${i}`,
      text: `text ${i}`,
      source_url: null,
      source_title: null,
      source_author: null,
      summary: `summary ${i}`,
      tags: ['a'],
      created_at: '2024-01-01',
    }))
    const page2 = Array.from({ length: 5 }, (_, i) => ({
      id: `h${20 + i}`,
      text: `text ${20 + i}`,
      source_url: null,
      source_title: null,
      source_author: null,
      summary: `summary ${20 + i}`,
      tags: ['b'],
      created_at: '2024-01-02',
    }))

    mockInvoke
      .mockResolvedValueOnce(page1)
      .mockResolvedValueOnce(page2)

    const { result } = renderHook(() => useHistory())
    await act(async () => await result.current.refresh())
    await waitFor(() => expect(result.current.entries.length).toBe(20))

    await act(async () => await result.current.loadMore())
    await waitFor(() => expect(result.current.entries.length).toBe(25))

    expect(result.current.hasMore).toBe(false)
    expect(mockInvoke).toHaveBeenLastCalledWith('get_history_paginated', { limit: 20, offset: 20 })
  })

  it('does not mutate entries on load failure', async () => {
    mockInvoke.mockRejectedValue(new Error('db locked'))

    const { result } = renderHook(() => useHistory())
    await act(async () => await result.current.refresh())

    expect(result.current.entries).toEqual([])
    expect(result.current.isLoading).toBe(false)
  })

  it('deletes and refreshes', async () => {
    mockInvoke.mockResolvedValue([])

    const { result } = renderHook(() => useHistory())
    await act(async () => await result.current.deleteHighlight('x'))

    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith('delete_highlight', { id: 'x' }))
    expect(mockInvoke).toHaveBeenCalledWith('get_history_paginated', { limit: 20, offset: 0 })
  })

  it('exposes entries and highlights as aliases', async () => {
    const page = [{ id: 'h1', text: 't1', source_url: null, source_title: null, source_author: null, summary: 's', tags: [], created_at: '2024-01-01' }]
    mockInvoke.mockResolvedValue(page)

    const { result } = renderHook(() => useHistory())
    await act(async () => await result.current.refresh())

    await waitFor(() => expect(result.current.entries.length).toBe(1))
    expect(result.current.highlights).toBe(result.current.entries)
  })
})
