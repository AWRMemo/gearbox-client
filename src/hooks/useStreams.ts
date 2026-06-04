import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import { writeText } from '../lib/clipboard'
import { useToast } from './useToast'
import type { StreamInfo, StreamHighlight } from '../types'

export interface UseStreamsReturn {
  streams: StreamInfo[]
  selectedStream: StreamInfo | null
  streamHighlights: StreamHighlight[]
  createTitle: string
  setCreateTitle: (t: string) => void
  createDesc: string
  setCreateDesc: (d: string) => void
  error: string | null
  shareLink: string | null
  copied: boolean
  loadStreams: () => Promise<void>
  loadStreamHighlights: (id: string) => Promise<void>
  selectStream: (s: StreamInfo) => Promise<void>
  createStream: () => Promise<void>
  deleteStream: (id: string) => Promise<void>
  addHighlight: (highlightId: string) => Promise<void>
  removeHighlight: (highlightId: string) => Promise<void>
  share: (channel: string) => Promise<void>
  copyLink: () => Promise<void>
  clearSelected: () => void
}

export function useStreams(): UseStreamsReturn {
  const { toast } = useToast()
  const [streams, setStreams] = useState<StreamInfo[]>([])
  const [selectedStream, setSelectedStream] = useState<StreamInfo | null>(null)
  const [streamHighlights, setStreamHighlights] = useState<StreamHighlight[]>([])
  const [createTitle, setCreateTitle] = useState('')
  const [createDesc, setCreateDesc] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [shareLink, setShareLink] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const loadStreams = useCallback(async () => {
    try {
      const result: StreamInfo[] = await invoke('list_my_streams')
      setStreams(result)
    } catch {
      // not critical
    }
  }, [])

  const loadStreamHighlights = useCallback(async (streamId: string) => {
    try {
      const result: StreamHighlight[] = await invoke('get_stream_highlights', { streamId })
      setStreamHighlights(result)
      setError(null)
    } catch (err) {
      const msg = String(err)
      setError(msg)
      toast({ message: msg, type: 'error' })
    }
  }, [toast])

  const selectStream = useCallback(
    async (s: StreamInfo) => {
      setSelectedStream(s)
      setShareLink(null)
      setCopied(false)
      setError(null)
      await loadStreamHighlights(s.id)
    },
    [loadStreamHighlights]
  )

  const createStream = useCallback(async () => {
    const title = createTitle.trim()
    if (!title) {
      const msg = 'Title is required'
      setError(msg)
      toast({ message: msg, type: 'error' })
      return
    }
    setError(null)
    try {
      const info: StreamInfo = await invoke('create_stream', {
        title,
        description: createDesc.trim(),
      })
      setStreams(prev => [info, ...prev])
      setCreateTitle('')
      setCreateDesc('')
      setSelectedStream(info)
      await loadStreamHighlights(info.id)
    } catch (err) {
      const msg = String(err)
      setError(msg)
      toast({ message: msg, type: 'error' })
    }
  }, [createTitle, createDesc, loadStreamHighlights, toast])

  const deleteStream = useCallback(
    async (streamId: string) => {
      try {
        await invoke('delete_stream', { streamId })
        setStreams(prev => prev.filter(s => s.id !== streamId))
        if (selectedStream?.id === streamId) {
          setSelectedStream(null)
          setStreamHighlights([])
          setShareLink(null)
        }
        setError(null)
        toast({ message: 'Stream deleted', type: 'success' })
      } catch (err) {
        const msg = String(err)
        setError(msg)
        toast({ message: msg, type: 'error' })
      }
    },
    [selectedStream, toast]
  )

  const addHighlight = useCallback(
    async (highlightId: string) => {
      if (!selectedStream) return
      try {
        await invoke('add_to_stream', { streamId: selectedStream.id, highlightId })
        await loadStreamHighlights(selectedStream.id)
        setError(null)
      } catch (err) {
        const msg = String(err)
        setError(msg)
        toast({ message: msg, type: 'error' })
      }
    },
    [selectedStream, loadStreamHighlights, toast]
  )

  const removeHighlight = useCallback(
    async (highlightId: string) => {
      if (!selectedStream) return
      try {
        await invoke('remove_from_stream', { streamId: selectedStream.id, highlightId })
        await loadStreamHighlights(selectedStream.id)
      } catch (err) {
        const msg = String(err)
        setError(msg)
        toast({ message: msg, type: 'error' })
      }
    },
    [selectedStream, loadStreamHighlights, toast]
  )

  const share = useCallback(
    async (channel: string) => {
      if (!selectedStream) return
      try {
        const link: string = await invoke('share_stream', {
          streamId: selectedStream.id,
          channel,
        })
        setShareLink(link)
      } catch (err) {
        const msg = String(err)
        setError(msg)
        toast({ message: msg, type: 'error' })
      }
    },
    [selectedStream, toast]
  )

  const copyLink = useCallback(async () => {
    if (!shareLink) return
    try {
      await writeText(shareLink)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      try {
        await navigator.clipboard.writeText(shareLink)
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
      } catch {
        const msg = 'Failed to copy to clipboard'
        setError(msg)
        toast({ message: msg, type: 'error' })
      }
    }
  }, [shareLink, toast])

  const clearSelected = useCallback(() => {
    setSelectedStream(null)
  }, [])

  return {
    streams,
    selectedStream,
    streamHighlights,
    createTitle,
    setCreateTitle,
    createDesc,
    setCreateDesc,
    error,
    shareLink,
    copied,
    loadStreams,
    loadStreamHighlights,
    selectStream,
    createStream,
    deleteStream,
    addHighlight,
    removeHighlight,
    share,
    copyLink,
    clearSelected,
  }
}
