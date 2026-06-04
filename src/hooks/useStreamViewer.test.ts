import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useStreamViewer } from './useStreamViewer'

vi.mock('../lib/tauri', () => ({ invoke: vi.fn() }))

import { invoke } from '../lib/tauri'
const mockInvoke = invoke as ReturnType<typeof vi.fn>

describe('useStreamViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('has empty initial state', () => {
    const { result } = renderHook(() => useStreamViewer())
    expect(result.current.stream).toBeNull()
    expect(result.current.highlights).toEqual([])
    expect(result.current.error).toBeNull()
    expect(result.current.curatorProfile).toBeNull()
    expect(result.current.isSubscribed).toBe(false)
  })

  it('loads stream and highlights', async () => {
    const stream = {
      id: 's-1',
      user_id: 'u-1',
      title: 'Test Stream',
      description: 'Desc',
      is_public: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    }
    const highlights = [
      { id: 'h-1', text: 'Text', summary: 'Summary', tags: ['tag'], source_url: null },
    ]

    mockInvoke.mockImplementation(async (cmd: string, _args?: Record<string, unknown>) => {
      if (cmd === 'get_stream') return stream
      if (cmd === 'get_stream_highlights') return highlights
      if (cmd === 'get_device_id') return 'u-1'
      if (cmd === 'log_stream_page_view') return undefined
      if (cmd === 'is_subscribed_to_stream') return false
      return undefined
    })

    const { result } = renderHook(() => useStreamViewer())
    await act(async () => await result.current.loadStream('s-1'))

    await waitFor(() => expect(result.current.stream).not.toBeNull())
    expect(result.current.stream?.title).toBe('Test Stream')
    expect(result.current.highlights.length).toBe(1)
    expect(result.current.error).toBeNull()
    expect(result.current.isSubscribed).toBe(false)
  })

  it('logs page view for non-owner', async () => {
    const stream = {
      id: 's-1',
      user_id: 'u-2',
      title: 'Another Stream',
      description: '',
      is_public: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    }

    mockInvoke.mockImplementation(async (cmd: string, _args?: Record<string, unknown>) => {
      if (cmd === 'get_stream') return stream
      if (cmd === 'get_stream_highlights') return []
      if (cmd === 'get_device_id') return 'u-1'
      if (cmd === 'log_stream_page_view') return undefined
      if (cmd === 'is_subscribed_to_stream') return false
      return undefined
    })

    const { result } = renderHook(() => useStreamViewer())
    await act(async () => await result.current.loadStream('s-1'))

    const pageViewCall = mockInvoke.mock.calls.find((call) => call[0] === 'log_stream_page_view')
    expect(pageViewCall).toBeDefined()
    expect((pageViewCall as unknown[])[1]).toMatchObject({
      streamId: 's-1',
      visitorId: null,
      isOwner: false,
    })
  })

  it('loads curator profile when available', async () => {
    const stream = {
      id: 's-1',
      user_id: 'u-2',
      title: 'Stream',
      description: '',
      is_public: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    }
    const profile = {
      user_id: 'u-2',
      email: 'curator@example.com',
      display_name: 'Curator',
      tier: 'pro',
      created_at: '2024-01-01',
    }

    mockInvoke.mockImplementation(async (cmd: string, _args?: Record<string, unknown>) => {
      if (cmd === 'get_stream') return stream
      if (cmd === 'get_stream_highlights') return []
      if (cmd === 'get_device_id') return 'u-1'
      if (cmd === 'log_stream_page_view') return undefined
      if (cmd === 'is_subscribed_to_stream') return false
      if (cmd === 'get_user_profile_by_id' && (_args as Record<string, unknown>)?.userId === 'u-2') return profile
      return undefined
    })

    const { result } = renderHook(() => useStreamViewer())
    await act(async () => await result.current.loadStream('s-1'))

    await waitFor(() => expect(result.current.curatorProfile).not.toBeNull())
    expect(result.current.curatorProfile?.email).toBe('curator@example.com')
  })

  it('toggles subscription on', async () => {
    const stream = {
      id: 's-1',
      user_id: 'u-2',
      title: 'Stream',
      description: '',
      is_public: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    }

    mockInvoke.mockImplementation(async (cmd: string, _args?: Record<string, unknown>) => {
      if (cmd === 'get_stream') return stream
      if (cmd === 'get_stream_highlights') return []
      if (cmd === 'get_device_id') return 'u-1'
      if (cmd === 'log_stream_page_view') return undefined
      if (cmd === 'is_subscribed_to_stream') return false
      if (cmd === 'subscribe_to_stream') return undefined
      return undefined
    })

    const { result } = renderHook(() => useStreamViewer())
    await act(async () => await result.current.loadStream('s-1'))
    expect(result.current.isSubscribed).toBe(false)

    await act(async () => await result.current.toggleSubscription())
    await waitFor(() => expect(result.current.isSubscribed).toBe(true))
    expect(mockInvoke).toHaveBeenCalledWith('subscribe_to_stream', { streamId: 's-1' })
  })

  it('toggles subscription off', async () => {
    const stream = {
      id: 's-1',
      user_id: 'u-2',
      title: 'Stream',
      description: '',
      is_public: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    }

    mockInvoke.mockImplementation(async (cmd: string, _args?: Record<string, unknown>) => {
      if (cmd === 'get_stream') return stream
      if (cmd === 'get_stream_highlights') return []
      if (cmd === 'get_device_id') return 'u-1'
      if (cmd === 'log_stream_page_view') return undefined
      if (cmd === 'is_subscribed_to_stream') return true
      if (cmd === 'unsubscribe_from_stream') return undefined
      return undefined
    })

    const { result } = renderHook(() => useStreamViewer())
    await act(async () => await result.current.loadStream('s-1'))
    expect(result.current.isSubscribed).toBe(true)

    await act(async () => await result.current.toggleSubscription())
    await waitFor(() => expect(result.current.isSubscribed).toBe(false))
    expect(mockInvoke).toHaveBeenCalledWith('unsubscribe_from_stream', { streamId: 's-1' })
  })

  it('sets error when loadStream fails', async () => {
    mockInvoke.mockRejectedValue(new Error('Stream not found'))
    const { result } = renderHook(() => useStreamViewer())
    await act(async () => await result.current.loadStream('bad-id'))
    await waitFor(() => expect(result.current.error).not.toBeNull())
    expect(result.current.error).toContain('Stream not found')
  })
})
