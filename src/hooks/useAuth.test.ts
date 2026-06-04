import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useAuth } from './useAuth'

vi.mock('../lib/tauri', () => ({ invoke: vi.fn() }))
vi.mock('./useToast', () => ({ useToast: () => ({ toast: vi.fn() }) }))

import { invoke } from '../lib/tauri'
const mockInvoke = invoke as ReturnType<typeof vi.fn>

describe('useAuth', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('starts unauthenticated', () => {
    mockInvoke.mockRejectedValue(new Error('no auth'))
    const { result } = renderHook(() => useAuth())
    expect(result.current.isAuthenticated).toBe(false)
    expect(result.current.email).toBeNull()
    expect(result.current.loading).toBe(false)
  })

  it('checks status on mount and sets authenticated', async () => {
    mockInvoke.mockResolvedValue({
      signed_in: true,
      email: 'user@example.com',
      server_url: 'http://localhost:3000',
    })
    const { result } = renderHook(() => useAuth())
    await waitFor(() => expect(result.current.isAuthenticated).toBe(true))
    expect(result.current.email).toBe('user@example.com')
  })

  it('logIn sets authenticated on success', async () => {
    mockInvoke.mockResolvedValueOnce({
      signed_in: false,
      email: null,
      server_url: null,
    })
    const { result } = renderHook(() => useAuth())
    await waitFor(() => expect(result.current.loading).toBe(false))

    mockInvoke.mockResolvedValueOnce(undefined)
    await act(async () => {
      await result.current.logIn('a@test.com', 'pw')
    })

    expect(result.current.isAuthenticated).toBe(true)
    expect(result.current.email).toBe('a@test.com')
  })

  it('logIn sets error on failure', async () => {
    mockInvoke.mockResolvedValueOnce({ signed_in: false })
    const { result } = renderHook(() => useAuth())
    await waitFor(() => expect(result.current.loading).toBe(false))

    mockInvoke.mockRejectedValueOnce(new Error('bad creds'))
    await act(async () => {
      await result.current.logIn('a@test.com', 'pw')
    })

    expect(result.current.isAuthenticated).toBe(false)
    expect(result.current.error).toContain('bad creds')
  })

  it('logOut sets unauthenticated', async () => {
    mockInvoke.mockResolvedValue({ signed_in: true, email: 'a@test.com' })
    const { result } = renderHook(() => useAuth())
    await waitFor(() => expect(result.current.isAuthenticated).toBe(true))

    mockInvoke.mockResolvedValueOnce(undefined)
    await act(async () => {
      await result.current.logOut()
    })

    expect(result.current.isAuthenticated).toBe(false)
    expect(result.current.email).toBeNull()
  })
})
