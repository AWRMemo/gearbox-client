import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { PaywallModal } from './PaywallModal'

vi.mock('../hooks', () => ({
  useToast: () => ({ toast: vi.fn() }),
}))

describe('PaywallModal', () => {
  it('renders nothing when trigger is null', () => {
    const { container } = render(<PaywallModal trigger={null} onDismiss={vi.fn()} />)
    expect(container.innerHTML).toBe('')
  })

  it('renders upgrade prompt for stream limit', () => {
    render(<PaywallModal trigger="free_stream_limit" onDismiss={vi.fn()} />)
    expect(screen.getByText('Upgrade to Pro')).toBeInTheDocument()
    expect(screen.getByText(/Free tier is limited to 3 Streams/)).toBeInTheDocument()
  })

  it('renders upgrade prompt for review limit', () => {
    render(<PaywallModal trigger="free_review_limit" onDismiss={vi.fn()} />)
    expect(screen.getByText(/Free tier is limited to 1 Review/)).toBeInTheDocument()
  })

  it('calls onDismiss when Maybe Later clicked', () => {
    const onDismiss = vi.fn()
    render(<PaywallModal trigger="free_stream_limit" onDismiss={onDismiss} />)
    fireEvent.click(screen.getByText('Maybe Later'))
    expect(onDismiss).toHaveBeenCalledTimes(1)
  })
})
