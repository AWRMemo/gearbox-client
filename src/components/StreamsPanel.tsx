import { EmptyState } from './EmptyState'
import { SkeletonRow } from './SkeletonRow'
import type { StreamInfo } from '../types'

interface StreamsPanelProps {
  streams: StreamInfo[]
  isLoading?: boolean
  createTitle: string
  onCreateTitleChange: (t: string) => void
  createDesc: string
  onCreateDescChange: (d: string) => void
  error: string | null
  onCreate: () => void
  onSelect: (s: StreamInfo) => void
  onDelete: (id: string) => void
}

export function StreamsPanel({
  streams,
  isLoading,
  createTitle,
  onCreateTitleChange,
  createDesc,
  onCreateDescChange,
  error,
  onCreate,
  onSelect,
  onDelete,
}: StreamsPanelProps) {
  return (
    <div>
      <h2>My Streams</h2>
      <div className="create-stream-form">
        <h3>Create New Stream</h3>
        <div className="controls" style={{ flexDirection: 'column', gap: '0.5rem' }}>
          <input
            type="text"
            value={createTitle}
            onChange={(e) => onCreateTitleChange(e.target.value)}
            placeholder="Title…"
            className="search-input"
            onKeyDown={(e) => {
              if (e.key === 'Enter') onCreate()
            }}
          />
          <input
            type="text"
            value={createDesc}
            onChange={(e) => onCreateDescChange(e.target.value)}
            placeholder="Description (optional)…"
            className="search-input"
            onKeyDown={(e) => {
              if (e.key === 'Enter') onCreate()
            }}
          />
          <button onClick={onCreate}>Create Stream</button>
        </div>
        {error && (
          <p style={{ color: 'var(--danger-text)', fontSize: '0.85rem', marginTop: '0.3rem' }}>{error}</p>
        )}
      </div>
      {isLoading && (
        <div style={{ marginTop: '1rem' }}>
          <SkeletonRow />
          <SkeletonRow />
          <SkeletonRow />
        </div>
      )}
      {!isLoading && streams.length === 0 && (
        <EmptyState
          title="No streams yet"
          description="Create your first Stream to curate and share highlights."
        />
      )}
      <div style={{ marginTop: '1rem' }}>
        {streams.map((s) => (
          <div
            key={s.id}
            className="result-card"
            style={{ cursor: 'pointer' }}
            onClick={() => onSelect(s)}
          >
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <div style={{ flex: 1 }}>
                <h3 style={{ color: 'var(--text-heading)', marginBottom: '0.2rem' }}>{s.title}</h3>
                {s.description && (
                  <p style={{ color: 'var(--text-secondary)', fontSize: '0.85rem' }}>{s.description}</p>
                )}
                <p style={{ color: 'var(--text-subtle)', fontSize: '0.75rem', marginTop: '0.3rem' }}>
                  Created {new Date(s.created_at).toLocaleDateString()}
                </p>
              </div>
              <button
                style={{
                  fontSize: '0.75rem',
                  padding: '0.3rem 0.6rem',
                  background: 'var(--danger-bg)',
                  marginLeft: '0.5rem',
                }}
                onClick={(e) => {
                  e.stopPropagation()
                  onDelete(s.id)
                }}
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
