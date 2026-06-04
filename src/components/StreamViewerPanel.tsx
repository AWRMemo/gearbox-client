import type { StreamInfo, StreamHighlight, UserProfile } from '../types'

interface StreamViewerPanelProps {
  stream: StreamInfo | null
  highlights: StreamHighlight[]
  error: string | null
  curatorProfile: UserProfile | null
  isSubscribed: boolean
  onToggleSubscription: () => void
}

export function StreamViewerPanel({
  stream,
  highlights,
  error,
  curatorProfile,
  isSubscribed,
  onToggleSubscription,
}: StreamViewerPanelProps) {
  return (
    <div>
      {error && (
        <div className="result-card" style={{ borderLeft: '3px solid var(--danger-text)' }}>{error}</div>
      )}
      {stream && (
        <>
          <h2>{stream.title}</h2>
          {stream.description && (
            <p style={{ color: 'var(--text-secondary)', fontSize: '0.9rem', marginBottom: '1rem' }}>{stream.description}</p>
          )}
          <p style={{ color: 'var(--text-secondary)', fontSize: '0.85rem', marginBottom: '1rem' }}>
            Curated by {curatorProfile?.email || 'Relay User'}
          </p>
          <div className="controls" style={{ marginBottom: '1rem' }}>
            <button
              className={
                isSubscribed ? 'subscribe-btn subscribed' : 'subscribe-btn'
              }
              onClick={onToggleSubscription}
            >
              {isSubscribed ? 'Unsubscribe' : 'Subscribe'}
            </button>
          </div>
          <div style={{ marginTop: '1rem' }}>
            {highlights.map((h) => (
              <div key={h.id} className="result-card">
                <p style={{ fontSize: '0.9rem', lineHeight: 1.4 }}>{h.summary}</p>
                <div className="tags" style={{ marginTop: '0.3rem' }}>
                  {h.tags.map((t, i) => (
                    <span key={i} className="tag">{t}</span>
                  ))}
                </div>
                <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '0.3rem' }}>
                  {h.text.substring(0, 100)}
                  {h.text.length > 100 ? '…' : ''}
                </p>
                {h.source_url && (
                  <p
                    style={{
                      fontSize: '0.75rem',
                      color: 'var(--text-subtle)',
                      marginTop: '0.2rem',
                    }}
                  >
                    {h.source_url}
                  </p>
                )}
              </div>
            ))}
            {highlights.length === 0 && (
              <p style={{ color: 'var(--text-subtle)', fontSize: '0.85rem' }}>No highlights.</p>
            )}
          </div>
        </>
      )}
    </div>
  )
}
