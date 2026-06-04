import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { useStreams } from './useStreams'

vi.mock('../lib/tauri', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '../lib/tauri'
const mockInvoke = invoke as ReturnType<typeof vi.fn>

describe('useStreams', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('has empty initial state', () => {
    const { result } = renderHook(() => useStreams())
    expect(result.current.streams).toEqual([])
    expect(result.current.selectedStream).toBeNull()
    expect(result.current.error).toBeNull()
  })

  it('loads streams successfully', async () => {
    const mockStreams = [
      { id: 's1', user_id: 'u1', title: 'AI Papers', description: '', is_public: true, created_at: '2024-01-01', updated_at: '2024-01-01' },
    ]
    mockInvoke.mockResolvedValue(mockStreams)

    const { result } = renderHook(() => useStreams())
    await act(async () => await result.current.loadStreams())

    expect(result.current.streams.length).toBe(1)
    expect(result.current.streams[0].title).toBe('AI Papers')
  })

  it('rejects create without title', async () => {
    const { result } = renderHook(() => useStreams())
    await act(async () => await result.current.createStream())
    expect(result.current.error).toBe('Title is required')
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('creates stream with title', async () => {
    const newStream = { id: 's2', user_id: 'u1', title: 'New Stream', description: '', is_public: true, created_at: '2024-01-01', updated_at: '2024-01-01' }
    mockInvoke.mockResolvedValue(newStream)

    const { result } = renderHook(() => useStreams())
    act(() => result.current.setCreateTitle('New Stream'))
    await act(async () => await result.current.createStream())

    await waitFor(() => expect(result.current.streams.length).toBe(1))
    expect(result.current.streams[0].id).toBe('s2')
    expect(result.current.selectedStream).not.toBeNull()
  })
})
