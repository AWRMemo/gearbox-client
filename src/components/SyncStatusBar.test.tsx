import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SyncStatusBar } from './SyncStatusBar'

vi.mock('../hooks', () => ({
  useAuth: vi.fn(),
  useSync: vi.fn(),
}))

import { useAuth, useSync } from '../hooks'
const mockUseAuth = useAuth as ReturnType<typeof vi.fn>
const mockUseSync = useSync as ReturnType<typeof vi.fn>

describe('SyncStatusBar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows not signed in when unauthenticated', () => {
    mockUseAuth.mockReturnValue({ isAuthenticated: false })
    mockUseSync.mockReturnValue({ syncStatus: null, loading: false, error: null, syncNow: vi.fn(), conflicts: [] })
    render(<SyncStatusBar />)
    expect(screen.getByText(/Not signed in/i)).toBeInTheDocument()
  })

  it('shows email and sync status when authenticated', () => {
    mockUseAuth.mockReturnValue({ isAuthenticated: true, email: 'a@test.com' })
    mockUseSync.mockReturnValue({
      syncStatus: { last_sync: '2026-05-20T12:00:00Z', status: 'idle', pending_conflicts: 0 },
      loading: false,
      error: null,
      syncNow: vi.fn(),
      conflicts: [],
    })
    render(<SyncStatusBar />)
    expect(screen.getByText(/a@test.com/)).toBeInTheDocument()
    expect(screen.getByText(/Last sync:/)).toBeInTheDocument()
    expect(screen.getByText(/Sync Now/i)).toBeInTheDocument()
  })

  it('shows conflict count when present', () => {
    mockUseAuth.mockReturnValue({ isAuthenticated: true, email: 'a@test.com' })
    mockUseSync.mockReturnValue({
      syncStatus: { last_sync: null, status: 'idle', pending_conflicts: 3 },
      loading: false,
      error: null,
      syncNow: vi.fn(),
      conflicts: [],
    })
    render(<SyncStatusBar />)
    expect(screen.getByText(/Conflicts: 3/)).toBeInTheDocument()
  })
})
