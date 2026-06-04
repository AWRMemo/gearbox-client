import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useToast, __resetToastsForTests } from './useToast'

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true })
  __resetToastsForTests()
})

afterEach(() => {
  vi.useRealTimers()
})

describe('useToast', () => {
  it('starts with empty toasts', () => {
    const { result } = renderHook(() => useToast())
    expect(result.current.toasts).toEqual([])
  })

  it('shows a toast', () => {
    const { result } = renderHook(() => useToast())
    act(() => result.current.toast({ message: 'Hello', type: 'info' }))
    expect(result.current.toasts.length).toBe(1)
    expect(result.current.toasts[0].message).toBe('Hello')
    expect(result.current.toasts[0].type).toBe('info')
  })

  it('auto-dismisses after 3 seconds with exit transition', () => {
    const { result } = renderHook(() => useToast())
    act(() => result.current.toast({ message: 'Auto', type: 'success' }))
    expect(result.current.toasts.length).toBe(1)
    act(() => vi.advanceTimersByTime(3000))
    // after auto-dismiss, toast is in exiting state but still in list until transition ends
    expect(result.current.toasts.length).toBe(1)
    expect(result.current.toasts[0].exiting).toBe(true)
    act(() => vi.advanceTimersByTime(350)) // exit transition + buffer
    expect(result.current.toasts.length).toBe(0)
  })

  it('dismisses manually', () => {
    const { result } = renderHook(() => useToast())
    act(() => result.current.toast({ message: 'Manual', type: 'error' }))
    const id = result.current.toasts[0].id
    act(() => result.current.dismiss(id))
    expect(result.current.toasts.length).toBe(1)
    expect(result.current.toasts[0].exiting).toBe(true)
    act(() => vi.advanceTimersByTime(350))
    expect(result.current.toasts.length).toBe(0)
  })

  it('shows multiple toasts', () => {
    const { result } = renderHook(() => useToast())
    act(() => result.current.toast({ message: 'One', type: 'info' }))
    act(() => result.current.toast({ message: 'Two', type: 'success' }))
    expect(result.current.toasts.length).toBe(2)
  })

  it('clears timer on manual dismiss', () => {
    const { result } = renderHook(() => useToast())
    act(() => result.current.toast({ message: 'Clear', type: 'info' }))
    const id = result.current.toasts[0].id
    act(() => result.current.dismiss(id))
    act(() => vi.advanceTimersByTime(5000))
    expect(result.current.toasts.length).toBe(0)
  })
})
