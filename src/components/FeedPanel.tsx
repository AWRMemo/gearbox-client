import { EmptyState } from './EmptyState'
import type { FeedHighlight } from '../types'

interface FeedPanelProps {
  feed: FeedHighlight[]
  error: string | null
}

export function FeedPanel({ feed, error }: FeedPanelProps) {
  return (
    <div>
      <h2>Subscriber Feed</h2>
      {error && (
        <div className="result-card" style={{ borderLeft: '3px solid var(--danger-text)' }}>
          {error}
        </div>
      )}
      {feed.length === 0 && !error && (
        <EmptyState
          title="Your feed is empty"
          description="Subscribe to Streams to see curated highlights here."
        />
      )}
      {feed.map((h) => (
        <div key={h.id} className="result-card">
          <p style={{ fontSize: '0.75rem', color: 'var(--accent)', marginBottom: '0.2rem' }}>
            {h.stream_title}
          </p>
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
            <p style={{ fontSize: '0.75rem', color: 'var(--text-subtle)', marginTop: '0.2rem' }}>
              {h.source_url}
            </p>
          )}
        </div>
      ))}
    </div>
  )
}
