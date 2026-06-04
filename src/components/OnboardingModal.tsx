import { useState, useEffect, useRef, useCallback } from 'react'

const SCREENS = [
  {
    title: 'Welcome to Gearbox Relay',
    body: 'Your personal knowledge base, powered entirely by on-device AI. Your highlights never leave your computer unless you choose to sync.',
  },
  {
    title: 'Capture Anything',
    body: 'Copy any text to instantly capture it. Relay enriches it with a smart summary and tags using a local AI model — no cloud, no tracking.',
  },
  {
    title: 'Curate Streams',
    body: 'Build Streams to organise and share highlight collections. Publish them with a single click. Anyone with the link can subscribe.',
  },
  {
    title: 'Sync & Privacy',
    body: 'Sign in to sync across devices. All data is encrypted end-to-end. We can\'t read your highlights, and neither can anyone else.',
  },
]

export function OnboardingModal() {
  const [visible, setVisible] = useState(() => {
    if (typeof window === 'undefined') return false
    return localStorage.getItem('relay_onboarding_seen') !== '1'
  })
  const [step, setStep] = useState(0)
  const overlayRef = useRef<HTMLDivElement>(null)

  const dismiss = useCallback(() => {
    setVisible(false)
    localStorage.setItem('relay_onboarding_seen', '1')
  }, [])

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') dismiss()
    }
    if (visible) {
      window.addEventListener('keydown', handleKey)
      return () => window.removeEventListener('keydown', handleKey)
    }
  }, [visible, dismiss])

  function handleOverlayClick(e: React.MouseEvent<HTMLDivElement>) {
    if (e.target === overlayRef.current) {
      dismiss()
    }
  }

  if (!visible) return null

  const screen = SCREENS[step]
  const isLast = step === SCREENS.length - 1

  return (
    <div
      ref={overlayRef}
      data-testid="onboarding-overlay"
      onClick={handleOverlayClick}
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0,0,0,0.6)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 2000,
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        style={{
          background: 'var(--bg-card)',
          padding: '2rem',
          borderRadius: '12px',
          maxWidth: '420px',
          width: '90%',
          boxShadow: '0 8px 24px rgba(0,0,0,0.4)',
        }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'center',
            gap: '0.4rem',
            marginBottom: '1.2rem',
          }}
        >
          {SCREENS.map((_, i) => (
            <span
              key={i}
              style={{
                width: '0.5rem',
                height: '0.5rem',
                borderRadius: '50%',
                background: i === step ? 'var(--accent)' : 'var(--text-muted)',
                transition: 'background 0.2s',
              }}
            />
          ))}
        </div>
        <h2 style={{ marginBottom: '1rem', color: 'var(--text-heading)', fontSize: '1.2rem' }}>
          {screen.title}
        </h2>
        <p
          style={{
            lineHeight: 1.5,
            color: 'var(--text-primary)',
            marginBottom: '1.5rem',
            fontSize: '0.9rem',
          }}
        >
          {screen.body}
        </p>
        <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem' }}>
          {step > 0 ? (
            <button
              onClick={() => setStep((s) => s - 1)}
              style={{ background: 'transparent', border: '1px solid var(--border)', color: 'var(--text-primary)' }}
            >
              Back
            </button>
          ) : (
            <button
              onClick={dismiss}
              style={{ background: 'transparent', border: '1px solid var(--border)', color: 'var(--text-secondary)' }}
            >
              Skip
            </button>
          )}
          <button
            onClick={() => {
              if (isLast) {
                dismiss()
              } else {
                setStep((s) => s + 1)
              }
            }}
          >
            {isLast ? 'Get Started' : 'Next'}
          </button>
        </div>
      </div>
    </div>
  )
}
