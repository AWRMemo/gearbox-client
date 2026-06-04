import { useState } from 'react'

interface ConnectionSuggestionData {
  source_highlight_id?: string
  target_summary?: string
  source_summary?: string
  bridging_sentence?: string
}

interface ConnectionSuggestionCardProps {
  suggestion: string
  highlightId: string
  onNavigate?: (targetId: string) => void
}

function parseSuggestion(raw: string): ConnectionSuggestionData | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as ConnectionSuggestionData
  } catch {
    return { bridging_sentence: raw }
  }
}

const DISMISSED_PREFIX = 'relay_suggestion_dismissed:'

export function ConnectionSuggestionCard({
  suggestion,
  highlightId,
  onNavigate,
}: ConnectionSuggestionCardProps) {
  const [dismissed, setDismissed] = useState(() => {
    try {
      return localStorage.getItem(DISMISSED_PREFIX + highlightId) === 'true'
    } catch {
      return false
    }
  })

  if (dismissed) return null

  const data = parseSuggestion(suggestion)
  if (!data) return null

  const handleDismiss = () => {
    try {
      localStorage.setItem(DISMISSED_PREFIX + highlightId, 'true')
    } catch {
      // localStorage may be unavailable
    }
    setDismissed(true)
  }

  return (
    <div className="result-card connection-suggestion-card" role="complementary" aria-label="Connection suggestion">
      <p style={{ fontSize: '0.75rem', color: 'var(--accent)', marginBottom: '0.3rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
        Connection Suggestion
      </p>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap' }}>
        <span style={{ fontSize: '0.85rem', color: 'var(--text-primary)' }}>
          {data.source_summary || data.bridging_sentence || 'Related'}
        </span>
        <span style={{ color: 'var(--text-secondary)' }}>→</span>
        <span style={{ fontSize: '0.85rem', color: 'var(--text-primary)' }}>
          {data.target_summary || (data.source_highlight_id ? 'View related' : 'Not yet captured')}
        </span>
      </div>
      <div style={{ marginTop: '0.4rem', display: 'flex', gap: '0.5rem' }}>
        {data.source_highlight_id && onNavigate && (
          <button
            className="suggestion-btn"
            onClick={() => onNavigate(data.source_highlight_id!)}
          >
            View
          </button>
        )}
        <button
          className="suggestion-btn suggestion-btn--dismiss"
          onClick={handleDismiss}
        >
          Dismiss
        </button>
      </div>
    </div>
  )
}
