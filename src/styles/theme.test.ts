import { describe, it, expect, beforeEach, vi } from 'vitest'
import { getStoredTheme, setStoredTheme, applyTheme } from './theme'

beforeEach(() => {
  localStorage.clear()
  document.documentElement.removeAttribute('data-theme')
  window.matchMedia = vi.fn().mockReturnValue({ matches: true })
})

describe('theme', () => {
  it('getStoredTheme returns dark when no preference stored', () => {
    const theme = getStoredTheme()
    expect(theme).toBe('dark')
  })

  it('setStoredTheme and getStoredTheme roundtrip', () => {
    setStoredTheme('light')
    expect(getStoredTheme()).toBe('light')
    setStoredTheme('dark')
    expect(getStoredTheme()).toBe('dark')
  })

  it('applyTheme sets data-theme attribute', () => {
    applyTheme('light')
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
    applyTheme('dark')
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
  })
})
