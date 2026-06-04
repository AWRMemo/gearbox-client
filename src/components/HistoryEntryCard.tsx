import { useState } from 'react'
import { SourceMeta } from './SourceMeta'
import type { HistoryEntry } from '../types'

interface HistoryEntryCardProps {
  entry: HistoryEntry
  isAdded: boolean
  selectedStreamTitle: string | null
  onAddToStream: (id: string) => void
}

export function HistoryEntryCard({ entry, isAdded, selectedStreamTitle, onAddToStream }: HistoryEntryCardProps) {
  const [isExpanded, setIsExpanded] = useState(false)
  const r = entry.result

  return (
    <div
      className="result-card"
      style={{ cursor: 'pointer' }}
      onClick={() => setIsExpanded(prev => !prev)}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div style={{ flex: 1 }}>
          <span className="time">{new Date(entry.timestamp).toLocaleTimeString()}</span>
          <p style={{ fontSize: '0.85rem', color: 'var(--text-primary)', marginTop: '0.2rem' }}>
            {isExpanded ? entry.text : entry.text.substring(0, 150)}
            {entry.text.length > 150 && !isExpanded ? '…' : ''}
          </p>
          {!isExpanded && (
            <>
              <p
                style={{
                  fontSize: '0.85rem',
                  color: 'var(--accent)',
                  marginTop: '0.3rem',
                  fontStyle: 'italic',
                }}
              >
                {r.summary}
              </p>
              {r.tags.length > 0 && (
                <div className="tags" style={{ marginTop: '0.3rem' }}>
                  {r.tags.map((tag, i) => (
                    <span key={i} className="tag">{tag}</span>
                  ))}
                </div>
              )}
            </>
          )}
          {isExpanded && (
            <>
              <h3 style={{ marginTop: '0.8rem' }}>Summary</h3>
              <p style={{ fontSize: '0.9rem', lineHeight: 1.4 }}>{r.summary}</p>
              {r.tags.length > 0 && (
                <>
                  <h3 style={{ marginTop: '0.8rem' }}>Tags</h3>
                  <div className="tags">{r.tags.map((tag, i) => <span key={i} className="tag">{tag}</span>)}</div>
                </>
              )}
              {r.connection_suggestion !== null && (
                <>
                  <h3 style={{ marginTop: '0.8rem' }}>Connection</h3>
                  <p>{r.connection_suggestion}</p>
                </>
              )}
            </>
          )}
          <SourceMeta {...r} />
        </div>
        {selectedStreamTitle && (
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: '0.3rem',
              marginLeft: '0.5rem',
            }}
          >
            <button
              style={{ fontSize: '0.75rem', padding: '0.3rem 0.6rem', opacity: isAdded ? 0.5 : 1 }}
              onClick={(e) => {
                e.stopPropagation()
                onAddToStream(r.id)
              }}
              disabled={isAdded}
            >
              {isAdded ? '✓' : '+'}
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
