import { useHistory } from '../hooks/useHistory'
import { EmptyState } from './EmptyState'
import { SkeletonRow } from './SkeletonRow'
import { ConnectionSuggestionCard } from './ConnectionSuggestionCard'

export function HistoryPanel() {
  const { entries, isLoading, hasMore, loadMore, deleteHighlight } = useHistory()

  if (isLoading && entries.length === 0) {
    return (
      <div className="panel">
        <h2>History</h2>
        <SkeletonRow />
        <SkeletonRow />
        <SkeletonRow />
      </div>
    )
  }

  if (!isLoading && entries.length === 0) {
    return (
      <div className="panel">
        <h2>History</h2>
        <EmptyState
          title="No highlights yet"
          description="Copy text to get started."
        />
      </div>
    )
  }

  return (
    <div className="panel">
      <h2>History</h2>
      {entries.map(h => (
        <div key={h.id}>
          <div className="result-card" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
            <div style={{ flex: 1 }}>
              <p style={{ fontSize: '0.85rem', color: 'var(--accent)', fontStyle: 'italic' }}>{h.summary}</p>
              {h.tags.length > 0 && (
                <div className="tags" style={{ marginTop: '0.3rem' }}>
                  {h.tags.map((tag, i) => (
                    <span key={i} className="tag">{tag}</span>
                  ))}
                </div>
              )}
              <p style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginTop: '0.3rem' }}>
                {new Date(h.created_at).toLocaleDateString()}
                {h.source_url && (
                  <>
                    {' · '}
                    <a href={h.source_url} target="_blank" rel="noreferrer" style={{ color: 'var(--text-secondary)' }}>
                      {h.source_title ?? h.source_url}
                    </a>
                  </>
                )}
              </p>
            </div>
            <button
              aria-label={`Delete highlight ${h.summary.slice(0, 30)}`}
              style={{ fontSize: '0.75rem', padding: '0.3rem 0.6rem', marginLeft: '0.5rem' }}
              onClick={() => {
                if (window.confirm('Delete this highlight?')) {
                  deleteHighlight(h.id)
                }
              }}
            >
              Delete
            </button>
          </div>
          {h.connection_suggestion && (
            <ConnectionSuggestionCard
              suggestion={h.connection_suggestion}
              highlightId={h.id}
            />
          )}
        </div>
      ))}
      {hasMore && (
        <div style={{ marginTop: '0.5rem' }}>
          <button onClick={loadMore} disabled={isLoading}>
            {isLoading ? 'Loading…' : 'Load more'}
          </button>
        </div>
      )}
    </div>
  )
}
