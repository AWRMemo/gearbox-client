import { useAuth, useSync } from '../hooks'

export function SyncStatusBar() {
  const auth = useAuth()
  const sync = useSync()

  if (!auth.isAuthenticated) {
    return (
      <div className="result-card" style={{ marginTop: '1rem' }}>
        <p style={{ fontSize: '0.9rem', color: 'var(--text-muted)' }}>Not signed in. Sync is disabled.</p>
      </div>
    )
  }

  const status = sync.syncStatus

  return (
    <div className="result-card" style={{ marginTop: '1rem' }}>
      <p style={{ fontSize: '0.9rem' }}>
        <strong>Signed in as</strong> {auth.email}
      </p>
      <p style={{ fontSize: '0.85rem', color: 'var(--text-muted)', marginTop: '0.3rem' }}>
        Last sync: {status?.last_sync ?? 'Never'} · Status:{' '}
        {sync.loading ? 'Syncing…' : status?.status ?? 'idle'}
      </p>
      {status && (status.pending_conflicts ?? 0) > 0 && (
        <p style={{ fontSize: '0.85rem', color: 'var(--danger-text)', marginTop: '0.3rem' }}>
          Conflicts: {status.pending_conflicts}
        </p>
      )}
      <button
        onClick={() => sync.syncNow()}
        disabled={sync.loading}
        style={{ marginTop: '0.5rem' }}
      >
        {sync.loading ? 'Syncing…' : 'Sync Now'}
      </button>
      {sync.error && (
        <p style={{ color: 'var(--danger-text)', fontSize: '0.85rem', marginTop: '0.3rem' }}>
          {sync.error}
        </p>
      )}
    </div>
  )
}
