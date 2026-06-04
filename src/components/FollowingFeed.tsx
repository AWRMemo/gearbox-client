import { useState, useEffect, useRef, startTransition } from 'react'
import { EmptyState } from './EmptyState'
import { SkeletonRow } from './SkeletonRow'

interface SubscribedStream {
  id: string
  user_id: string
  title: string
  description: string
  is_public: boolean
  created_at: string
  updated_at: string
}

interface FollowingFeedProps {
  onNavigate?: (streamId: string) => void
}

async function fetchSubscriptions(): Promise<SubscribedStream[]> {
  const { invoke } = await import('../lib/tauri')
  return invoke('get_subscriptions')
}

export function FollowingFeed({ onNavigate }: FollowingFeedProps) {
  const [streams, setStreams] = useState<SubscribedStream[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [unsubscribingId, setUnsubscribingId] = useState<string | null>(null)
  const cancelledRef = useRef(false)

  const load = async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await fetchSubscriptions()
      if (!cancelledRef.current) setStreams(result)
    } catch (err) {
      if (!cancelledRef.current) setError(String(err))
    } finally {
      if (!cancelledRef.current) setIsLoading(false)
    }
  }

  useEffect(() => {
    cancelledRef.current = false
    startTransition(() => { load() })
    return () => { cancelledRef.current = true }
  }, [])

  const handleUnsubscribe = async (streamId: string, title: string) => {
    if (!window.confirm(`Unsubscribe from "${title}"?`)) return
    setUnsubscribingId(streamId)
    try {
      const { invoke } = await import('../lib/tauri')
      await invoke('unsubscribe_from_stream', { streamId })
      setStreams((prev) => prev.filter((s) => s.id !== streamId))
    } catch (err) {
      setError(String(err))
    } finally {
      setUnsubscribingId(null)
    }
  }

  if (isLoading && streams.length === 0) {
    return (
      <div className="panel">
        <h2>Following</h2>
        <SkeletonRow />
        <SkeletonRow />
        <SkeletonRow />
      </div>
    )
  }

  return (
    <div className="panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2>Following</h2>
        <button onClick={load} disabled={isLoading} style={{ fontSize: '0.8rem' }}>
          {isLoading ? '…' : 'Refresh'}
        </button>
      </div>
      {error && (
        <div className="result-card" style={{ borderLeft: '3px solid var(--danger-text)', marginTop: '0.5rem' }}>
          {error}
        </div>
      )}
      {streams.length === 0 && !isLoading && !error && (
        <EmptyState
          title="Not following any Streams"
          description="Subscribe to Streams shared by other users to see them here."
        />
      )}
      {streams.map((s) => (
        <div key={s.id} className="result-card following-card">
          <div
            role="button"
            tabIndex={0}
            className="following-card__content"
            onClick={() => onNavigate?.(s.id)}
            onKeyDown={(e) => { if (e.key === 'Enter') onNavigate?.(s.id) }}
            aria-label={`View stream: ${s.title}`}
          >
            <h3 style={{ fontSize: '1rem', color: 'var(--accent)', marginBottom: '0.2rem' }}>{s.title}</h3>
            {s.description && (
              <p style={{ fontSize: '0.85rem', color: 'var(--text-primary)' }}>{s.description}</p>
            )}
            <p style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginTop: '0.3rem' }}>
              Updated {new Date(s.updated_at).toLocaleDateString()}
            </p>
          </div>
          <button
            className="following-card__unsubscribe"
            onClick={() => handleUnsubscribe(s.id, s.title)}
            disabled={unsubscribingId === s.id}
          >
            {unsubscribingId === s.id ? '…' : 'Unsubscribe'}
          </button>
        </div>
      ))}
    </div>
  )
}
