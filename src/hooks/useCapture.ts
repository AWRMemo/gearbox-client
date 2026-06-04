import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import { readText } from '../lib/clipboard'
import { createChannel } from '../lib/channel'
import { useToast } from './useToast'
import type { CaptureResult } from '../types'

export interface UseCaptureReturn {
  streamingSummary: string
  streamingTags: string[]
  streamingConnection: string | null
  isStreaming: boolean
  lastResult: CaptureResult | null
  error: string | null
  expandedEntryId: string | null
  addedToStream: Set<string>
  handleCapture: () => Promise<void>
  setExpandedEntryId: (id: string | null) => void
  markAddedToStream: (id: string) => void
  toast: (opts: { message: string; type?: 'error' | 'success' | 'info' }) => void
}

export function useCapture(): UseCaptureReturn {
  const { toast } = useToast()
  const [streamingSummary, setStreamingSummary] = useState('')
  const [streamingTags, setStreamingTags] = useState<string[]>([])
  const [streamingConnection, setStreamingConnection] = useState<string | null>(null)
  const [isStreaming, setIsStreaming] = useState(false)
  const [lastResult, setLastResult] = useState<CaptureResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [expandedEntryId, setExpandedEntryId] = useState<string | null>(null)
  const [addedToStream, setAddedToStream] = useState<Set<string>>(new Set())

  const handleCapture = useCallback(async () => {
    setError(null)
    setStreamingSummary('')
    setStreamingTags([])
    setStreamingConnection(null)
    setLastResult(null)
    setIsStreaming(true)

    try {
      const text = await readText()
      if (!text || text.trim().length === 0) {
        const msg = 'Clipboard is empty'
        setError(msg)
        toast({ message: msg, type: 'error' })
        setIsStreaming(false)
        return
      }

      const channel = createChannel<Record<string, unknown>>()
      let currentTags: string[] = []
      let currentConnection: string | null = null

      channel.onmessage = (msg: Record<string, unknown>) => {
        const field = msg.field as string
        const value = msg.value
        switch (field) {
          case 'summary': {
            setStreamingSummary(prev => prev + (value as string))
            break
          }
          case 'tag': {
            currentTags = [...currentTags, value as string]
            setStreamingTags([...currentTags])
            break
          }
          case 'connection': {
            currentConnection = value as string | null
            setStreamingConnection(currentConnection)
            break
          }
          case 'done': {
            setIsStreaming(false)
            break
          }
        }
      }

      const result: CaptureResult = await invoke('enrich_clipboard', { text, onEvent: channel })
      setLastResult(result)
    } catch (err) {
      const msg = String(err)
      setError(msg)
      toast({ message: msg, type: 'error' })
    } finally {
      setIsStreaming(false)
    }
  }, [toast])

  const markAddedToStream = useCallback((id: string) => {
    setAddedToStream(prev => new Set(prev).add(id))
  }, [])

  return {
    streamingSummary,
    streamingTags,
    streamingConnection,
    isStreaming,
    lastResult,
    error,
    expandedEntryId,
    addedToStream,
    handleCapture,
    setExpandedEntryId,
    markAddedToStream,
    toast,
  }
}
