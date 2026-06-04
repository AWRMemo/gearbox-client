import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { useSettings } from './useSettings'

vi.mock('../lib/tauri', () => ({
  invoke: vi.fn(),
}))

vi.mock('./useToast', () => ({
  useToast: () => ({ toast: vi.fn() }),
}))

import { invoke } from '../lib/tauri'
const mockInvoke = invoke as ReturnType<typeof vi.fn>

describe('useSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('has empty initial state', () => {
    const { result } = renderHook(() => useSettings())
    expect(result.current.profile).toBeNull()
    expect(result.current.emailInput).toBe('')
    expect(result.current.message).toBeNull()
    expect(result.current.autoCapture).toBe(true)
  })

  it('loads profile successfully', async () => {
    const mockProfile = {
      user_id: 'u-123',
      email: 'test@example.com',
      display_name: null,
      tier: 'free',
      created_at: '2024-01-01',
    }
    mockInvoke.mockResolvedValue(mockProfile)

    const { result } = renderHook(() => useSettings())
    await act(async () => await result.current.loadProfile())

    expect(result.current.profile).not.toBeNull()
    expect(result.current.profile!.email).toBe('test@example.com')
    expect(result.current.emailInput).toBe('test@example.com')
  })

  it('loads auto-capture state', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_auto_capture_enabled') return false
      return null
    })

    const { result } = renderHook(() => useSettings())
    await act(async () => await result.current.loadProfile())

    expect(result.current.autoCapture).toBe(false)
    expect(mockInvoke).toHaveBeenCalledWith('get_auto_capture_enabled')
  })

  it('saves email successfully', async () => {
    mockInvoke.mockResolvedValue(undefined)

    const { result } = renderHook(() => useSettings())
    act(() => result.current.setEmailInput('new@example.com'))
    await act(async () => await result.current.saveEmail())

    await waitFor(() => expect(result.current.message).toContain('saved'))
    expect(mockInvoke).toHaveBeenCalledWith('set_user_email', { email: 'new@example.com' })
  })

  it('sets error message on save failure', async () => {
    mockInvoke.mockRejectedValue(new Error('db locked'))

    const { result } = renderHook(() => useSettings())
    act(() => result.current.setEmailInput('fail@example.com'))
    await act(async () => await result.current.saveEmail())

    await waitFor(() => expect(result.current.message).toContain('db locked'))
  })

  it('setAutoCapture calls invoke and updates state', async () => {
    mockInvoke.mockResolvedValue(undefined)

    const { result } = renderHook(() => useSettings())
    expect(result.current.autoCapture).toBe(true)

    await act(async () => await result.current.setAutoCapture(false))

    expect(mockInvoke).toHaveBeenCalledWith('set_auto_capture_enabled', { enabled: false })
    expect(result.current.autoCapture).toBe(false)
  })
})
