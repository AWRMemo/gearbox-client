import { useState, type FormEvent } from 'react'
import { useAuth } from '../hooks'
import type { UserProfile } from '../types'

interface AuthFormProps {
  profile: UserProfile | null
  emailInput: string
  onEmailChange: (e: string) => void
  message: string | null
  onSaveEmail: () => void
}

export function AuthForm({
  profile,
  emailInput,
  onEmailChange,
  message,
  onSaveEmail,
}: AuthFormProps) {
  const auth = useAuth()
  const [mode, setMode] = useState<'signup' | 'login'>('signup')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    if (mode === 'signup') {
      if (password !== confirmPassword) {
        return
      }
      await auth.createAccount(emailInput, password)
    } else {
      await auth.logIn(emailInput, password)
    }
  }

  if (auth.isAuthenticated) {
    return (
      <div className="result-card" style={{ marginTop: '1rem' }}>
        <p style={{ fontSize: '0.9rem' }}>
          Signed in as <strong>{auth.email}</strong>
        </p>
        <button
          onClick={() => auth.logOut()}
          disabled={auth.loading}
          style={{ marginTop: '0.5rem' }}
        >
          {auth.loading ? 'Signing out…' : 'Sign out'}
        </button>
      </div>
    )
  }

  return (
    <div className="result-card" style={{ marginTop: '1rem' }}>
      <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '0.5rem' }}>
        <button
          className={mode === 'signup' ? 'primary' : ''}
          onClick={() => setMode('signup')}
        >
          Sign Up
        </button>
        <button
          className={mode === 'login' ? 'primary' : ''}
          onClick={() => setMode('login')}
        >
          Log In
        </button>
      </div>
      {profile && (
        <div style={{ marginBottom: '0.8rem' }}>
          <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
            Device ID: {profile.user_id.substring(0, 8)}…
          </p>
          <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginTop: '0.3rem' }}>
            Tier: <strong>{profile.tier === 'pro' ? 'Pro' : 'Free'}</strong>
          </p>
          <div style={{ marginTop: '0.8rem' }}>
            <label
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
        </div>
      )}
      <form onSubmit={handleSubmit}>
        <input
          type="email"
          value={emailInput}
          onChange={(e) => onEmailChange(e.target.value)}
          placeholder="Email"
          required
          className="search-input"
          style={{ marginBottom: '0.5rem' }}
        />
        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Password"
          required
          className="search-input"
          style={{ marginBottom: '0.5rem' }}
        />
        {mode === 'signup' && (
          <input
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            placeholder="Confirm password"
            required
            className="search-input"
            style={{ marginBottom: '0.5rem' }}
          />
        )}
        {auth.error && (
          <p style={{ color: 'var(--danger-text)', fontSize: '0.85rem', marginBottom: '0.5rem' }}>
            {auth.error}
          </p>
        )}
        <button type="submit" disabled={auth.loading}>
          {auth.loading
            ? mode === 'signup'
              ? 'Creating account…'
              : 'Signing in…'
            : mode === 'signup'
            ? 'Create Account'
            : 'Sign In'}
        </button>
      </form>
    </div>
  )
}
