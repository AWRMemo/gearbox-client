import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useCapture } from './useCapture'

vi.mock('../lib/tauri', () => ({ invoke: vi.fn() }))
vi.mock('../lib/clipboard', () => ({ readText: vi.fn() }))
vi.mock('../lib/channel', () => ({ createChannel: vi.fn() }))

import { invoke } from '../lib/tauri'
import { readText } from '../lib/clipboard'
import { createChannel } from '../lib/channel'

const mockInvoke = invoke as ReturnType<typeof vi.fn>
const mockReadText = readText as ReturnType<typeof vi.fn>
const mockCreateChannel = createChannel as ReturnType<typeof vi.fn>

describe('useCapture', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('has empty initial state', () => {
    const { result } = renderHook(() => useCapture())
    expect(result.current.streamingSummary).toBe('')
    expect(result.current.streamingTags).toEqual([])
    expect(result.current.streamingConnection).toBeNull()
    expect(result.current.isStreaming).toBe(false)
    expect(result.current.lastResult).toBeNull()
    expect(result.current.error).toBeNull()
  })

  it('captures clipboard text and streams enrichment', async () => {
    mockReadText.mockResolvedValue('Test highlight text')
    mockCreateChannel.mockReturnValue({ onmessage: null })

    mockInvoke.mockImplementation(async (cmd: string, args: Record<string, unknown>) => {
      if (cmd === 'enrich_clipboard') {
        const channel = args.onEvent as { onmessage: ((msg: Record<string, unknown>) => void) | null }
        await new Promise((r) => setTimeout(r, 5))
        if (channel.onmessage) {
          channel.onmessage({ field: 'summary', value: 'T' })
          channel.onmessage({ field: 'summary', value: 'e' })
          channel.onmessage({ field: 'summary', value: 's' })
          channel.onmessage({ field: 'summary', value: 't' })
          channel.onmessage({ field: 'tag', value: 'test' })
          channel.onmessage({ field: 'done' })
        }
        return {
          id: 'hl-1',
          summary: 'Test',
          tags: ['test'],
          connection_suggestion: null,
          source_url: null,
          source_title: null,
          source_author: null,
        }
      }
    })

    const { result } = renderHook(() => useCapture())
    await act(async () => {
      await result.current.handleCapture()
    })

    await waitFor(() => expect(result.current.isStreaming).toBe(false))
    expect(result.current.streamingSummary).toBe('Test')
    expect(result.current.streamingTags).toEqual(['test'])
    expect(result.current.lastResult).toEqual({
      id: 'hl-1',
      summary: 'Test',
      tags: ['test'],
      connection_suggestion: null,
      source_url: null,
      source_title: null,
      source_author: null,
    })
    expect(result.current.error).toBeNull()
  })

  it('sets error when clipboard is empty', async () => {
    mockReadText.mockResolvedValue('   ')
    const { result } = renderHook(() => useCapture())
    await act(async () => await result.current.handleCapture())
    expect(result.current.error).toBe('Clipboard is empty')
    expect(result.current.isStreaming).toBe(false)
  })

  it('sets error when invoke fails', async () => {
    mockReadText.mockResolvedValue('Some text')
    mockCreateChannel.mockReturnValue({ onmessage: null })
    mockInvoke.mockRejectedValue(new Error('Inference failed'))

    const { result } = renderHook(() => useCapture())
    await act(async () => await result.current.handleCapture())
    await waitFor(() => expect(result.current.error).not.toBeNull())
    expect(result.current.error).toContain('Inference failed')
    expect(result.current.isStreaming).toBe(false)
  })

  it('appends multiple tags during streaming', async () => {
    mockReadText.mockResolvedValue('Multi tag text')
    mockCreateChannel.mockReturnValue({ onmessage: null })

    mockInvoke.mockImplementation(async (cmd: string, args: Record<string, unknown>) => {
      if (cmd === 'enrich_clipboard') {
        const channel = args.onEvent as { onmessage: ((msg: Record<string, unknown>) => void) | null }
        await new Promise((r) => setTimeout(r, 5))
        if (channel.onmessage) {
          channel.onmessage({ field: 'tag', value: 'rust' })
          channel.onmessage({ field: 'tag', value: 'tauri' })
          channel.onmessage({ field: 'done' })
        }
        return {
          id: 'hl-2',
          summary: 'Multi tag text',
          tags: ['rust', 'tauri'],
          connection_suggestion: null,
          source_url: null,
          source_title: null,
          source_author: null,
        }
      }
    })

    const { result } = renderHook(() => useCapture())
    await act(async () => {
      await result.current.handleCapture()
    })

    await waitFor(() => expect(result.current.isStreaming).toBe(false))
    expect(result.current.streamingTags).toEqual(['rust', 'tauri'])
  })

  it('sets connection suggestion when streamed', async () => {
    mockReadText.mockResolvedValue('Connected text')
    mockCreateChannel.mockReturnValue({ onmessage: null })

    mockInvoke.mockImplementation(async (cmd: string, args: Record<string, unknown>) => {
      if (cmd === 'enrich_clipboard') {
        const channel = args.onEvent as { onmessage: ((msg: Record<string, unknown>) => void) | null }
        await new Promise((r) => setTimeout(r, 5))
        if (channel.onmessage) {
          channel.onmessage({ field: 'connection', value: 'Links to prior concept.' })
          channel.onmessage({ field: 'done' })
        }
        return {
          id: 'hl-3',
          summary: 'Connected text',
          tags: [],
          connection_suggestion: { source_highlight_id: 'prev', bridging_sentence: 'Links to prior concept.' },
          source_url: null,
          source_title: null,
          source_author: null,
        }
      }
    })

    const { result } = renderHook(() => useCapture())
    await act(async () => {
      await result.current.handleCapture()
    })

    await waitFor(() => expect(result.current.isStreaming).toBe(false))
    expect(result.current.streamingConnection).toBe('Links to prior concept.')
  })

  it('tracks added-to-stream state per entry', async () => {
    mockReadText.mockResolvedValue('Trackable text')
    mockCreateChannel.mockReturnValue({ onmessage: null })
    mockInvoke.mockResolvedValue({
      id: 'hl-4',
      summary: 'Trackable text',
      tags: [],
      connection_suggestion: null,
      source_url: null,
      source_title: null,
      source_author: null,
    })

    const { result } = renderHook(() => useCapture())
    await act(async () => {
      await result.current.handleCapture()
    })

    expect(result.current.addedToStream.has('hl-4')).toBe(false)
    act(() => result.current.markAddedToStream('hl-4'))
    expect(result.current.addedToStream.has('hl-4')).toBe(true)
  })
})
