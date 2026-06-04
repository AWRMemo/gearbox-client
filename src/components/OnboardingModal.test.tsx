import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { OnboardingModal } from './OnboardingModal'

describe('OnboardingModal', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('renders when localStorage key is absent', () => {
    render(<OnboardingModal />)
    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByText('Welcome to Gearbox Relay')).toBeInTheDocument()
  })

  it('does not render when already seen', () => {
    localStorage.setItem('relay_onboarding_seen', '1')
    const { container } = render(<OnboardingModal />)
    expect(container.firstChild).toBeNull()
  })

  it('dismisses on Get Started click and writes localStorage', () => {
    render(<OnboardingModal />)
    // advance through all 4 slides
    fireEvent.click(screen.getByText('Next'))
    fireEvent.click(screen.getByText('Next'))
    fireEvent.click(screen.getByText('Next'))
    fireEvent.click(screen.getByText('Get Started'))
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    expect(localStorage.getItem('relay_onboarding_seen')).toBe('1')
  })

  it('dismisses on overlay click', () => {
    render(<OnboardingModal />)
    fireEvent.click(screen.getByTestId('onboarding-overlay'))
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('dismisses on Escape key', () => {
    render(<OnboardingModal />)
    fireEvent.keyDown(document, { key: 'Escape' })
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })
})
