import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { useFeed } from './useFeed'

vi.mock('../lib/tauri', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '../lib/tauri'
const mockInvoke = invoke as ReturnType<typeof vi.fn>

describe('useFeed', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('has empty initial state', () => {
    const { result } = renderHook(() => useFeed())
    expect(result.current.feed).toEqual([])
    expect(result.current.error).toBeNull()
  })

  it('loads feed successfully', async () => {
    const mockFeed = [
      {
        id: 'f1',
        text: 'Rust memory safety.',
        summary: 'Rust eliminates data races.',
        tags: ['rust'],
        source_url: null,
        stream_id: 's1',
        stream_title: 'Systems Programming',
      },
    ]
    mockInvoke.mockResolvedValue(mockFeed)

    const { result } = renderHook(() => useFeed())
    await act(async () => await result.current.loadFeed())

    expect(result.current.feed.length).toBe(1)
    expect(result.current.feed[0].stream_title).toBe('Systems Programming')
    expect(result.current.error).toBeNull()
  })

  it('sets error on failed load', async () => {
    mockInvoke.mockRejectedValue(new Error('network'))

    const { result } = renderHook(() => useFeed())
    await act(async () => await result.current.loadFeed())

    await waitFor(() => expect(result.current.error).not.toBeNull())
    expect(result.current.error).toContain('network')
  })
})
