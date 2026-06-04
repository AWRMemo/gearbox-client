import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import { useToast } from '../hooks/useToast'
import { EmptyState } from './EmptyState'
import { SkeletonRow } from './SkeletonRow'

interface ReviewItem {
  highlight_id: string
  text: string
  summary: string
  tags: string[]
  ease_factor: number
  interval_days: number
  next_review_at: string
  review_count: number
}

interface ReviewSessionResponse {
  items: ReviewItem[]
  total_due: number
}

export function ReviewPanel() {
  const { toast } = useToast()
  const [session, setSession] = useState<ReviewSessionResponse | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [currentIndex, setCurrentIndex] = useState(0)
  const [showText, setShowText] = useState(false)

  const loadSession = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    setCurrentIndex(0)
    setShowText(false)
    try {
      const result: ReviewSessionResponse = await invoke('get_review_session', { limit: 20 })
      setSession(result)
      if (result.items.length === 0) {
        toast({ message: 'All caught up! No highlights due for review.', type: 'info' })
      }
    } catch (err) {
      setError(String(err))
      toast({ message: String(err), type: 'error' })
    } finally {
      setIsLoading(false)
    }
  }, [toast])

  const handleGrade = useCallback(async (grade: number) => {
    if (!session || currentIndex >= session.items.length) return
    const item = session.items[currentIndex]
    try {
      await invoke('grade_review_item', { highlightId: item.highlight_id, grade })
      if (currentIndex + 1 >= session.items.length) {
        toast({ message: 'Session complete! Great work.', type: 'success' })
        setSession(null)
      } else {
        setCurrentIndex(prev => prev + 1)
        setShowText(false)
      }
    } catch (err) {
      toast({ message: String(err), type: 'error' })
    }
  }, [session, currentIndex, toast])

  if (isLoading && !session) {
    return (
      <div className="panel">
        <h2>Review</h2>
        <SkeletonRow /><SkeletonRow /><SkeletonRow />
      </div>
    )
  }

  if (error && !session) {
    return (
      <div className="panel">
        <h2>Review</h2>
        <div className="result-card" style={{ borderLeft: '3px solid var(--danger-text)' }}>{error}</div>
      </div>
    )
  }

  if (!session || session.items.length === 0) {
    return (
      <div className="panel">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <h2>Review</h2>
          <button onClick={loadSession} disabled={isLoading} style={{ fontSize: '0.8rem' }}>
            {isLoading ? '…' : 'Start Review'}
          </button>
        </div>
        <EmptyState title="All caught up!" description="Come back later for more review sessions." />
      </div>
    )
  }

  const item = session.items[currentIndex]
  const progress = `${currentIndex + 1} / ${session.items.length}`

  return (
    <div className="panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2>Review</h2>
        <span style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>{progress}</span>
      </div>
      <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: '0.5rem' }}>
        {session.total_due} total due · {session.items.length} in this session
      </p>
      <div
        className="result-card review-card"
        onClick={() => setShowText(!showText)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') setShowText(!showText) }}
        aria-label={showText ? 'Tap to show summary' : 'Tap to show full text'}
      >
        {!showText ? (
          <>
            <p className="summary-text">{item.summary}</p>
            {item.tags.length > 0 && (
              <div className="tags" style={{ marginTop: '0.5rem' }}>
                {item.tags.map((t, i) => <span key={i} className="tag">{t}</span>)}
              </div>
            )}
            <p style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginTop: '0.5rem' }}>
              Tap to reveal full text
            </p>
          </>
        ) : (
          <>
            <p style={{ fontSize: '0.9rem', lineHeight: 1.5 }}>{item.text}</p>
            <p style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginTop: '0.5rem' }}>
              Tap to hide
            </p>
          </>
        )}
      </div>
      <div className="review-grades" style={{ marginTop: '1rem' }}>
        <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: '0.5rem' }}>How well did you remember?</p>
        <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
          <button className="grade-btn grade-btn--again" onClick={() => handleGrade(0)}>Again</button>
          <button className="grade-btn" onClick={() => handleGrade(2)}>Hard</button>
          <button className="grade-btn grade-btn--good" onClick={() => handleGrade(3)}>Good</button>
          <button className="grade-btn grade-btn--easy" onClick={() => handleGrade(5)}>Easy</button>
        </div>
      </div>
    </div>
  )
}
