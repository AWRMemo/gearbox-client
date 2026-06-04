import { useState } from 'react'
import { invoke } from '../lib/tauri'
import { useToast, useAuth, useModelStatus } from '../hooks'
import { EmptyState, AuthForm, SyncStatusBar, SyncConflictPanel } from '.'
import type { UserProfile } from '../types'

interface SettingsPanelProps {
  profile: UserProfile | null
  emailInput: string
  onEmailChange: (e: string) => void
  message: string | null
  onSaveEmail: () => void
  autoCapture: boolean
  onAutoCaptureChange: (v: boolean) => void
  telemetryEnabled: boolean
  onTelemetryChange: (v: boolean) => void
  theme: 'light' | 'dark'
  onThemeChange: (mode: 'light' | 'dark') => void
}

export function SettingsPanel({
  profile,
  emailInput,
  onEmailChange,
  message,
  onSaveEmail,
  autoCapture,
  onAutoCaptureChange,
  telemetryEnabled,
  onTelemetryChange,
  theme,
  onThemeChange,
}: SettingsPanelProps) {
  const { toast: showToast } = useToast()
  const auth = useAuth()
  const model = useModelStatus()
  const [exporting, setExporting] = useState(false)
  const [clearing, setClearing] = useState(false)
  const [reDownloading, setReDownloading] = useState(false)
  const [exportDateFrom, setExportDateFrom] = useState('')
  const [exportDateTo, setExportDateTo] = useState('')

  const handleExport = async () => {
    setExporting(true)
    try {
      const path: string = await invoke('export_local_data', {
        dateFrom: exportDateFrom || null,
        dateTo: exportDateTo || null,
      })
      showToast({ message: `Export saved to ${path}`, type: 'success' })
    } catch (err) {
      showToast({ message: `Export failed: ${String(err)}`, type: 'error' })
    } finally {
      setExporting(false)
    }
  }

  const handleClear = async () => {
    if (!window.confirm('This will permanently delete all local data. Are you sure?')) return
    setClearing(true)
    try {
      await invoke('clear_local_data')
      showToast({ message: 'All local data cleared. Please restart the app.', type: 'success' })
    } catch (err) {
      showToast({ message: `Clear failed: ${String(err)}`, type: 'error' })
    } finally {
      setClearing(false)
    }
  }

  const handleResetOnboarding = () => {
    localStorage.removeItem('relay_onboarding_seen')
    showToast({ message: 'Onboarding reset. Restart the app to see it again.', type: 'info' })
  }

  const handleReDownloadEmbedding = async () => {
    setReDownloading(true)
    try {
      await invoke('re_download_embedding_model')
      showToast({ message: 'Embedding model re-download started. Restart to apply.', type: 'success' })
      model.load()
    } catch (err) {
      showToast({ message: `Re-download failed: ${String(err)}`, type: 'error' })
    } finally {
      setReDownloading(false)
    }
  }

  const embeddingUnavailable = model.status?.embedding_available === false

  return (
    <div>
      <h2>Settings</h2>
      {profile && (
        <div className="result-card">
          <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
            Device ID: {profile.user_id.substring(0, 8)}…
          </p>
          <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginTop: '0.3rem' }}>
            Tier: <strong>{profile.tier === 'pro' ? 'Pro' : 'Free'}</strong>
            {profile.tier !== 'pro' && (
              <span
                style={{
                  color: 'var(--accent)',
                  fontSize: '0.8rem',
                  marginLeft: '0.5rem',
                }}
              >
                Unlimited Streams with Pro
              </span>
            )}
          </p>
          <div style={{ marginTop: '0.8rem' }}>
            <label
              htmlFor="settings-email"
              style={{
                fontSize: '0.85rem',
                color: 'var(--text-muted)',
                display: 'block',
                marginBottom: '0.3rem',
              }}
            >
              Email (optional, shown as curator name)
            </label>
            <div className="controls" style={{ marginBottom: 0 }}>
              <input
                id="settings-email"
                type="email"
                value={emailInput}
                onChange={(e) => onEmailChange(e.target.value)}
                placeholder="your@email.com"
                className="search-input"
              />
              <button onClick={onSaveEmail}>Save</button>
            </div>
          </div>
          {message && (
            <p
              style={{ color: 'var(--accent)', fontSize: '0.85rem', marginTop: '0.3rem' }}
            >
              {message}
            </p>
          )}
          <hr style={{ borderColor: 'var(--border)', margin: '1rem 0' }} />
          <label
            htmlFor="settings-auto-capture"
            style={{
              fontSize: '0.85rem',
              color: 'var(--text-muted)',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              cursor: 'pointer',
              marginBottom: '0.8rem',
            }}
          >
            <input
              id="settings-auto-capture"
              type="checkbox"
              checked={autoCapture}
              onChange={(e) => onAutoCaptureChange(e.target.checked)}
            />
            Auto-capture clipboard on copy
          </label>
          <label
            htmlFor="settings-telemetry"
            style={{
              fontSize: '0.85rem',
              color: 'var(--text-muted)',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              cursor: 'pointer',
              marginBottom: '0.8rem',
            }}
          >
            <input
              id="settings-telemetry"
              type="checkbox"
              checked={telemetryEnabled}
              onChange={(e) => onTelemetryChange(e.target.checked)}
            />
            Send anonymous crash and performance data to help improve Relay
          </label>
          <label
            htmlFor="settings-theme"
            style={{
              fontSize: '0.85rem',
              color: 'var(--text-muted)',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              cursor: 'pointer',
              marginBottom: '0.8rem',
            }}
          >
            <input
              id="settings-theme"
              type="checkbox"
              checked={theme === 'dark'}
              onChange={(e) => onThemeChange(e.target.checked ? 'dark' : 'light')}
            />
            Dark mode
          </label>
          {embeddingUnavailable && (
            <div
              style={{
                fontSize: '0.85rem',
                color: 'var(--danger-text)',
                marginBottom: '0.8rem',
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem',
              }}
            >
              <span>Search vectors: unavailable</span>
              <button
                onClick={handleReDownloadEmbedding}
                disabled={reDownloading}
                style={{ fontSize: '0.75rem', padding: '0.2rem 0.5rem' }}
              >
                {reDownloading ? 'Re-downloading…' : 'Re-download'}
              </button>
            </div>
          )}
          <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '0.5rem', fontSize: '0.8rem' }}>
            <label style={{ color: 'var(--text-muted)' }}>
              From:
              <input
                type="date"
                value={exportDateFrom}
                onChange={(e) => setExportDateFrom(e.target.value)}
                style={{
                  marginLeft: '0.3rem',
                  background: 'var(--input-bg)',
                  color: 'var(--input-text)',
                  border: '1px solid var(--border)',
                  padding: '0.2rem 0.4rem',
                  borderRadius: '4px',
                  fontSize: '0.8rem',
                }}
              />
            </label>
            <label style={{ color: 'var(--text-muted)' }}>
              To:
              <input
                type="date"
                value={exportDateTo}
                onChange={(e) => setExportDateTo(e.target.value)}
                style={{
                  marginLeft: '0.3rem',
                  background: 'var(--input-bg)',
                  color: 'var(--input-text)',
                  border: '1px solid var(--border)',
                  padding: '0.2rem 0.4rem',
                  borderRadius: '4px',
                  fontSize: '0.8rem',
                }}
              />
            </label>
          </div>
          <div className="controls" style={{ marginBottom: '0.8rem' }}>
            <button onClick={handleExport} disabled={exporting}>
              {exporting ? 'Exporting…' : 'Export my data'}
            </button>
            <button onClick={handleClear} disabled={clearing} style={{ background: 'var(--danger-bg)' }}>
              {clearing ? 'Clearing…' : 'Clear all data'}
            </button>
          </div>
          <button
            onClick={handleResetOnboarding}
            style={{
              fontSize: '0.8rem',
              background: 'transparent',
              border: '1px solid var(--border)',
              color: 'var(--text-secondary)',
            }}
          >
            Reset onboarding
          </button>
        </div>
      )}
      {!profile && (
        <EmptyState
          title="No profile yet"
          description="Set your email to enable sync across devices."
        />
      )}
      <AuthForm
        profile={profile}
        emailInput={emailInput}
        onEmailChange={onEmailChange}
        message={message}
        onSaveEmail={onSaveEmail}
      />
      {auth.isAuthenticated && (
        <>
          <SyncStatusBar />
          <SyncConflictPanel />
        </>
      )}
    </div>
  )
}
