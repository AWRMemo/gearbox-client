import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { SyncConflictPanel } from './SyncConflictPanel'

vi.mock('../hooks', () => ({
  useSync: vi.fn(),
}))

vi.mock('./useToast', () => ({
  useToast: () => ({ toast: vi.fn() }),
}))

import { useSync } from '../hooks'
const mockUseSync = useSync as ReturnType<typeof vi.fn>

describe('SyncConflictPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows no conflicts message', () => {
    mockUseSync.mockReturnValue({
      conflicts: [],
      loading: false,
      error: null,
      resolveConflict: vi.fn(),
      refresh: vi.fn(),
    })
    render(<SyncConflictPanel />)
    expect(screen.getByText(/No unresolved conflicts/i)).toBeInTheDocument()
  })

  it('renders conflict cards', () => {
    mockUseSync.mockReturnValue({
      conflicts: [
        {
          id: 'c1',
          record_type: 'highlight',
          record_id: 'hl-abc',
          local_version: '{"summary":"local"}',
          remote_version: '{"summary":"remote"}',
          created_at: '2026-05-20T12:00:00Z',
        },
      ],
      loading: false,
      error: null,
      resolveConflict: vi.fn(),
      refresh: vi.fn(),
    })
    render(<SyncConflictPanel />)
    expect(screen.getByText(/highlight/)).toBeInTheDocument()
    expect(screen.getByText(/Keep Local/i)).toBeInTheDocument()
    expect(screen.getByText(/Accept Remote/i)).toBeInTheDocument()
  })

  it('calls resolveConflict on button click', async () => {
    const mockResolve = vi.fn().mockResolvedValue(undefined)
    mockUseSync.mockReturnValue({
      conflicts: [
        {
          id: 'c1',
          record_type: 'highlight',
          record_id: 'hl-abc',
          local_version: '{"summary":"local"}',
          remote_version: '{"summary":"remote"}',
          created_at: '2026-05-20T12:00:00Z',
        },
      ],
      loading: false,
      error: null,
      resolveConflict: mockResolve,
      refresh: vi.fn().mockResolvedValue(undefined),
    })
    render(<SyncConflictPanel />)
    fireEvent.click(screen.getByText(/Keep Local/i))
    await waitFor(() => expect(mockResolve).toHaveBeenCalledWith('c1', 'keep_local'))
  })
})
