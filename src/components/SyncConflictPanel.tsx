import { useSync } from '../hooks'

export function SyncConflictPanel() {
  const { conflicts, loading, error, resolveConflict, refresh } = useSync()

  return (
    <div style={{ marginTop: '1rem' }}>
      <h3 style={{ fontSize: '1rem', marginBottom: '0.5rem' }}>Sync Conflicts</h3>
      {loading && <p style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Loading conflicts…</p>}
      {error && <p style={{ color: 'var(--danger-text)', fontSize: '0.85rem' }}>{error}</p>}
      {conflicts.length === 0 && !loading && (
        <p style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>No unresolved conflicts.</p>
      )}
      {conflicts.map((c) => (
        <div key={c.id} className="result-card" style={{ marginBottom: '0.5rem' }}>
          <p style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>
            {c.record_type} · {c.record_id.substring(0, 12)}…
          </p>
          <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '0.3rem' }}>
            Local: {c.local_version ? `${c.local_version.substring(0, 60)}…` : 'none'}
          </p>
          <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
            Remote: {c.remote_version ? `${c.remote_version.substring(0, 60)}…` : 'none'}
          </p>
          <div className="controls" style={{ marginTop: '0.5rem', marginBottom: 0 }}>
            <button
              onClick={() => resolveConflict(c.id, 'keep_local')}
              disabled={loading}
              style={{ background: 'var(--button-bg)' }}
            >
              Keep Local
            </button>
            <button
              onClick={() => resolveConflict(c.id, 'accept_remote')}
              disabled={loading}
              style={{ background: 'var(--button-bg)' }}
            >
              Accept Remote
            </button>
          </div>
        </div>
      ))}
      {conflicts.length > 0 && (
        <button onClick={() => refresh()} disabled={loading} style={{ marginTop: '0.5rem' }}>
          Refresh
        </button>
      )}
    </div>
  )
}
