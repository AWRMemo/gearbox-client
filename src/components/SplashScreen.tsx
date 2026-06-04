export function SplashScreen() {
  return (
    <div className="splash-screen" role="status" aria-label="Starting Relay">
      <div className="splash-content">
        <div className="splash-gear">
          <svg width="64" height="64" viewBox="0 0 64 64">
            <circle cx="32" cy="32" r="12" fill="none" stroke="var(--text-secondary)" strokeWidth="4" />
            <circle cx="32" cy="32" r="4" fill="#FFB000" />
            {[0, 45, 90, 135, 180, 225, 270, 315].map((angle, i) => (
              <rect
                key={angle}
                x={30} y={i % 2 === 0 ? 6 : 10}
                width="4" height={i % 2 === 0 ? 10 : 6}
                fill="#FFB000"
                transform={`rotate(${angle}, 32, 32)`}
                opacity={0.6}
              />
            ))}
          </svg>
        </div>
        <p className="splash-text" style={{ color: 'var(--text-primary)', fontSize: '16px', marginTop: '16px' }}>Starting your relay…</p>
        <div className="splash-progress">
          <div className="splash-bar" />
        </div>
      </div>
    </div>
  );
}
