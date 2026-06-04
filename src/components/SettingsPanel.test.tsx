import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { SettingsPanel } from './SettingsPanel'

vi.mock('../hooks', () => ({
  useSettings: () => ({ profile: null, emailInput: '', setEmailInput: vi.fn(), message: null, loadProfile: vi.fn(), saveEmail: vi.fn(), autoCapture: true, setAutoCapture: vi.fn(), telemetryOptOut: false, setTelemetryOptOut: vi.fn() }),
  useToast: () => ({ toast: vi.fn() }),
  useAuth: () => ({ isAuthenticated: false, email: null, loading: false, error: null, logIn: vi.fn(), logOut: vi.fn(), createAccount: vi.fn() }),
  useModelStatus: () => ({ status: null, load: vi.fn() }),
}))

vi.mock('../lib/tauri', () => ({
  invoke: vi.fn(),
}))

function defaultProps(overrides: Record<string, unknown> = {}) {
  return {
    profile: { user_id: 'u-test', email: null, display_name: null, tier: 'free', created_at: '2024-01-01' },
    emailInput: '',
    onEmailChange: vi.fn(),
    message: null,
    onSaveEmail: vi.fn(),
    autoCapture: false,
    onAutoCaptureChange: vi.fn(),
    telemetryEnabled: false,
    onTelemetryChange: vi.fn(),
    theme: 'dark' as const,
    onThemeChange: vi.fn(),
    ...overrides,
  }
}

describe('SettingsPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders profile info', () => {
    render(
      <SettingsPanel
        {...defaultProps({
          profile: { user_id: 'u-abc-123', email: 'user@example.com', display_name: null, tier: 'free', created_at: '2024-01-01' },
          emailInput: 'user@example.com',
        })}
      />
    )
    expect(screen.getAllByText(/Device ID:/).length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByText(/Free/).length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByDisplayValue('user@example.com').length).toBeGreaterThanOrEqual(1)
  })

  it('renders auto-capture checkbox', () => {
    render(
      <SettingsPanel
        {...defaultProps({
          profile: { user_id: 'u-abc-123', email: null, display_name: null, tier: 'pro', created_at: '2024-01-01' },
          autoCapture: true,
        })}
      />
    )
    const checkbox = screen.getByLabelText(/Auto-capture clipboard on copy/i)
    expect(checkbox).toBeInTheDocument()
    expect((checkbox as HTMLInputElement).checked).toBe(true)
  })

  it('renders onboarding reset button', () => {
    render(<SettingsPanel {...defaultProps()} />)
    expect(screen.getByText('Reset onboarding')).toBeInTheDocument()
  })

  it('calls setAutoCapture when checkbox is toggled', async () => {
    const mockToggle = vi.fn()
    render(
      <SettingsPanel
        {...defaultProps({
          autoCapture: false,
          onAutoCaptureChange: mockToggle,
        })}
      />
    )
    const checkbox = screen.getByLabelText(/Auto-capture clipboard on copy/i)
    fireEvent.click(checkbox)
    await waitFor(() => expect(mockToggle).toHaveBeenCalledWith(true))
  })
})
