import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { AuthForm } from './AuthForm'

vi.mock('../hooks', () => ({
  useAuth: vi.fn(),
}))

vi.mock('./useToast', () => ({
  useToast: () => ({ toast: vi.fn() }),
}))

import { useAuth } from '../hooks'
const mockUseAuth = useAuth as ReturnType<typeof vi.fn>

describe('AuthForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders sign-up form when unauthenticated', () => {
    mockUseAuth.mockReturnValue({
      isAuthenticated: false,
      email: null,
      serverUrl: null,
      loading: false,
      error: null,
      createAccount: vi.fn(),
      logIn: vi.fn(),
      logOut: vi.fn(),
      checkStatus: vi.fn(),
    })
    render(
      <AuthForm
        profile={null}
        emailInput=""
        onEmailChange={vi.fn()}
        message={null}
        onSaveEmail={vi.fn()}
      />
    )
    expect(screen.getByText(/Sign Up/i)).toBeInTheDocument()
    expect(screen.getByPlaceholderText(/Email/i)).toBeInTheDocument()
  })

  it('switches to login tab', () => {
    mockUseAuth.mockReturnValue({
      isAuthenticated: false,
      email: null,
      serverUrl: null,
      loading: false,
      error: null,
      createAccount: vi.fn(),
      logIn: vi.fn(),
      logOut: vi.fn(),
      checkStatus: vi.fn(),
    })
    render(
      <AuthForm
        profile={null}
        emailInput=""
        onEmailChange={vi.fn()}
        message={null}
        onSaveEmail={vi.fn()}
      />
    )
    fireEvent.click(screen.getByText(/Log In/i))
    expect(screen.getByText(/Sign In/i)).toBeInTheDocument()
  })

  it('calls createAccount on sign-up submit', async () => {
    const mockCreate = vi.fn().mockResolvedValue(undefined)
    mockUseAuth.mockReturnValue({
      isAuthenticated: false,
      email: null,
      serverUrl: null,
      loading: false,
      error: null,
      createAccount: mockCreate,
      logIn: vi.fn(),
      logOut: vi.fn(),
      checkStatus: vi.fn(),
    })
    render(
      <AuthForm
        profile={null}
        emailInput="a@test.com"
        onEmailChange={vi.fn()}
        message={null}
        onSaveEmail={vi.fn()}
      />
    )
    const pwInput = screen.getByPlaceholderText('Password')
    fireEvent.change(pwInput, { target: { value: 'secret' } })
    const confirmInput = screen.getByPlaceholderText('Confirm password')
    fireEvent.change(confirmInput, { target: { value: 'secret' } })
    fireEvent.click(screen.getByText(/Create Account/i))
    await waitFor(() => expect(mockCreate).toHaveBeenCalledWith('a@test.com', 'secret'))
  })

  it('shows sign-out button when authenticated', () => {
    mockUseAuth.mockReturnValue({
      isAuthenticated: true,
      email: 'user@example.com',
      serverUrl: null,
      loading: false,
      error: null,
      createAccount: vi.fn(),
      logIn: vi.fn(),
      logOut: vi.fn(),
      checkStatus: vi.fn(),
    })
    render(
      <AuthForm
        profile={null}
        emailInput=""
        onEmailChange={vi.fn()}
        message={null}
        onSaveEmail={vi.fn()}
      />
    )
    expect(screen.getByText(/Sign out/i)).toBeInTheDocument()
  })
})
