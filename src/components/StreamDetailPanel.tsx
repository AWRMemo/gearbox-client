import type { StreamInfo, StreamHighlight } from '../types'

interface StreamDetailPanelProps {
  stream: StreamInfo
  highlights: StreamHighlight[]
  shareLink: string | null
  copied: boolean
  error: string | null
  onBack: () => void
  onShare: (channel: string) => void
  onCopyLink: () => void
  onRemove: (id: string) => void
}

export function StreamDetailPanel({
  stream,
  highlights,
  shareLink,
  copied,
  error,
  onBack,
  onShare,
  onCopyLink,
  onRemove,
}: StreamDetailPanelProps) {
  return (
    <div>
      <button className="back-btn" onClick={onBack}>← Back</button>
      <h2>{stream.title}</h2>
      {stream.description && (
        <p style={{ color: 'var(--text-secondary)', fontSize: '0.9rem', marginBottom: '0.5rem' }}>{stream.description}</p>
      )}
      <div className="controls" style={{ marginTop: '0.5rem' }}>
        <button onClick={() => onShare('clipboard')}>Share</button>
        {shareLink && (
          <button onClick={onCopyLink}>{copied ? 'Copied!' : 'Copy Link'}</button>
        )}
      </div>
      {shareLink && (
        <div className="result-card" style={{ marginTop: '0.5rem' }}>
          <p
            style={{ fontSize: '0.85rem', color: 'var(--accent)', wordBreak: 'break-all' }}
          >
            {shareLink}
          </p>
        </div>
      )}
      {error && (
        <p style={{ color: 'var(--danger-text)', fontSize: '0.85rem', marginTop: '0.3rem' }}>{error}</p>
      )}
      <h3 style={{ marginTop: '1rem' }}>Highlights ({highlights.length})</h3>
      {highlights.length === 0 && (
        <p style={{ color: 'var(--text-subtle)', fontSize: '0.85rem' }}>No highlights yet.</p>
      )}
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
          <button
            style={{ marginTop: '0.4rem', fontSize: '0.8rem', background: 'var(--danger-bg)' }}
            onClick={() => onRemove(h.id)}
          >
            Remove
          </button>
        </div>
      ))}
    </div>
  )
}
