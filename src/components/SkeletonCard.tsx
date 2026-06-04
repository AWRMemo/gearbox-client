export function SkeletonCard() {
  return (
    <div className="skeleton result-card" style={{ padding: '1rem' }}>
      <div
        className="skeleton"
        style={{
          height: '1.2rem',
          width: '40%',
          borderRadius: '4px',
          marginBottom: '1rem',
        }}
      />
      <div
        className="skeleton"
        style={{
          height: '0.9rem',
          borderRadius: '4px',
          marginBottom: '0.5rem',
        }}
      />
      <div
        className="skeleton"
        style={{
          height: '0.9rem',
          width: '80%',
          borderRadius: '4px',
          marginBottom: '0.5rem',
        }}
      />
      <div
        className="skeleton"
        style={{
          height: '0.9rem',
          width: '60%',
          borderRadius: '4px',
        }}
      />
    </div>
  )
}
