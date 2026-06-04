import type { SearchResult } from '../types'

interface SearchResultCardProps {
  result: SearchResult
  selectedIndex?: number
  index?: number
  onAddToStream?: (id: string) => void
  selectedStream?: { title: string } | null
  onSelect?: (id: string) => void
}

export function SearchResultCard({
  result,
  selectedIndex,
  index,
  onAddToStream,
  selectedStream,
  onSelect,
}: SearchResultCardProps) {
  const isSelected = selectedIndex !== undefined && index !== undefined && selectedIndex === index

  const confidenceLabel =
    result.score >= 0.8 ? 'high' : result.score >= 0.5 ? 'medium' : 'low'

  return (
    <div
      className={`result-card${isSelected ? ' result-card--selected' : ''}`}
      onClick={() => onSelect?.(result.id)}
      onKeyDown={(e) => { if (e.key === 'Enter') onSelect?.(result.id) }}
      role="button"
      tabIndex={0}
      aria-label={`Search result: ${result.summary}`}
    >
      <p className="summary-text">{result.summary}</p>
      {result.tags.length > 0 && (
        <div className="tags" style={{ marginTop: '0.3rem' }}>
          {result.tags.map((t, i) => (
            <span key={i} className="tag">{t}</span>
          ))}
        </div>
      )}
      <p className="result-card__meta">
        {result.text.substring(0, 80)}{result.text.length > 80 ? '…' : ''}
      </p>
      <div className="result-card__footer">
        <span className={`confidence-badge confidence-badge--${confidenceLabel}`}>
          {confidenceLabel} match
        </span>
        {result.score !== 0 && (
          <span className="result-card__score">
            {(result.score * 100).toFixed(0)}%
          </span>
        )}
      </div>
      {selectedStream && onAddToStream && (
        <button
          className="add-to-stream-btn"
          onClick={(e) => { e.stopPropagation(); onAddToStream(result.id) }}
        >
          Add to {selectedStream.title}
        </button>
      )}
    </div>
  )
}
