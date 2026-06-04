interface EmptyStateProps {
  icon?: React.ReactNode
  title: string
  description: string
}

const DefaultIcon = (
  <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
    <polyline points="14 2 14 8 20 8" />
  </svg>
)

export function EmptyState({ icon, title, description }: EmptyStateProps) {
  return (
    <div
      style={{
        textAlign: 'center',
        padding: '2rem 1rem',
        color: 'var(--text-secondary)',
      }}
    >
      <div
        style={{
          marginBottom: '0.8rem',
          fontSize: '2rem',
          display: 'flex',
          justifyContent: 'center',
        }}
      >
        {icon ?? DefaultIcon}
      </div>
      <h3 style={{ color: 'var(--text-muted)', fontSize: '1rem', marginBottom: '0.4rem' }}>
        {title}
      </h3>
      <p style={{ fontSize: '0.85rem', lineHeight: 1.4 }}>{description}</p>
    </div>
  )
}
