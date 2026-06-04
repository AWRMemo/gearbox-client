import { SourceMeta } from './SourceMeta'
import { EmptyState } from './EmptyState'
import { SkeletonCard } from './SkeletonCard'
import type { CaptureResult } from '../types'
import type { UseModelStatusReturn } from '../hooks/useModelStatus'

interface CapturePanelProps {
  streamingSummary: string
  streamingTags: string[]
  streamingConnection: string | null
  isStreaming: boolean
  lastResult: CaptureResult | null
  error: string | null
  onCapture: () => void
  modelStatus?: UseModelStatusReturn['status']
}

export function CapturePanel({
  streamingSummary,
  streamingTags,
  streamingConnection,
  isStreaming,
  lastResult,
  error,
  onCapture,
  modelStatus,
}: CapturePanelProps) {
  return (
    <>
      <div className="controls">
        <button onClick={onCapture} disabled={isStreaming}>
          {isStreaming ? (
            <>
              Enriching
              <span
                className="pulse-dot"
                style={{ marginLeft: '0.3rem', display: 'inline-block' }}
              />
            </>
          ) : (
            'Capture'
          )}
        </button>
        {modelStatus && (
          <span
            style={{
              fontSize: '0.75rem',
              padding: '0.2rem 0.5rem',
              borderRadius: '4px',
              marginLeft: '0.5rem',
              alignSelf: 'center',
              whiteSpace: 'nowrap',
              ...(modelStatus.loaded
                ? {
                    color: 'var(--model-ready-text)',
                    border: '1px solid var(--model-ready-border)',
                    background: 'var(--model-ready-bg)',
                  }
                : {
                    color: 'var(--model-downloading-text)',
                    border: '1px solid var(--model-downloading-border)',
                    background: 'var(--model-downloading-bg)',
                  }),
            }}
          >
            {modelStatus.loaded
              ? `🧠 Local AI ready${modelStatus.model_name ? ` (${modelStatus.model_name})` : ''}`
              : '⏳ Downloading AI model…'}
          </span>
        )}
      </div>
      {error && (
        <div className="result-card" style={{ borderLeft: '3px solid var(--danger-text)' }}>
          {error}
        </div>
      )}
      {isStreaming && !streamingSummary && <SkeletonCard />}
      {(streamingSummary || streamingTags.length > 0) && (
        <div className="result-card">
          <h3>Summary</h3>
          <p className="summary-text">
            {streamingSummary}
            {isStreaming && <span className="cursor">|</span>}
          </p>
          {streamingTags.length > 0 && (
            <>
              <h3 style={{ marginTop: '0.8rem' }}>Tags</h3>
              <div className="tags">
                {streamingTags.map((t, i) => (
                  <span key={i} className="tag">
                    {t}
                  </span>
                ))}
              </div>
            </>
          )}
          {isStreaming && <p className="streaming-indicator">Streaming enrichment…</p>}
          {streamingConnection !== null && (
            <>
              <h3 style={{ marginTop: '0.8rem' }}>Connection</h3>
              <p>{streamingConnection}</p>
            </>
          )}
          {lastResult && lastResult.connection_suggestion === null && (
            <>
              <h3 style={{ marginTop: '0.8rem' }}>Connection</h3>
              <p style={{ color: 'var(--text-secondary)' }}>No connections yet</p>
            </>
          )}
        </div>
      )}
      {lastResult && <SourceMeta {...lastResult} />}
      {!isStreaming && !lastResult && !error && !streamingSummary && (
        <EmptyState
          title="Nothing captured yet"
          description="Copy text from anywhere, then click Capture to enrich it with AI."
        />
      )}
    </>
  )
}
