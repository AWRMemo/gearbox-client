import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import type { StreamInfo, StreamHighlight, UserProfile } from '../types'

export interface UseStreamViewerReturn {
  stream: StreamInfo | null
  highlights: StreamHighlight[]
  error: string | null
  curatorProfile: UserProfile | null
  isSubscribed: boolean
  loadStream: (streamId: string) => Promise<void>
  toggleSubscription: () => Promise<void>
}

export function useStreamViewer(): UseStreamViewerReturn {
  const [stream, setStream] = useState<StreamInfo | null>(null)
  const [highlights, setHighlights] = useState<StreamHighlight[]>([])
  const [error, setError] = useState<string | null>(null)
  const [curatorProfile, setCuratorProfile] = useState<UserProfile | null>(null)
  const [isSubscribed, setIsSubscribed] = useState(false)

  const checkSubscription = useCallback(async (streamId: string) => {
    try {
      const result: boolean = await invoke('is_subscribed_to_stream', { streamId })
      setIsSubscribed(result)
    } catch {
      setIsSubscribed(false)
    }
  }, [])

  const loadStream = useCallback(async (streamId: string) => {
    setError(null)
    setCuratorProfile(null)
    try {
      const info: StreamInfo = await invoke('get_stream', { streamId })
      setStream(info)
      const hls: StreamHighlight[] = await invoke('get_stream_highlights', { streamId })
      setHighlights(hls)
      const deviceId: string = await invoke('get_device_id')
      await invoke('log_stream_page_view', {
        streamId,
        visitorId: null,
        isOwner: info.user_id === deviceId,
      })
      await checkSubscription(streamId)
      try {
        const cp: UserProfile = await invoke('get_user_profile_by_id', {
          userId: info.user_id,
        })
        setCuratorProfile(cp)
      } catch {
        // curator has no profile yet
      }
    } catch (err) {
      setError(String(err))
    }
  }, [checkSubscription])

  const toggleSubscription = useCallback(async () => {
    if (!stream) return
    try {
      if (isSubscribed) {
        await invoke('unsubscribe_from_stream', { streamId: stream.id })
        setIsSubscribed(false)
      } else {
        await invoke('subscribe_to_stream', { streamId: stream.id })
        setIsSubscribed(true)
      }
    } catch (err) {
      setError(String(err))
    }
  }, [stream, isSubscribed])

  return { stream, highlights, error, curatorProfile, isSubscribed, loadStream, toggleSubscription }
}
