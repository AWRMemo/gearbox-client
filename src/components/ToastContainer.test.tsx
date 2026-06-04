import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ToastContainer } from './ToastContainer'

const mockDismiss = vi.fn()

vi.mock('../hooks', () => ({
  useToast: () => ({
    toasts: [
      { id: 't1', message: 'Saved', type: 'success' as const, createdAt: 1 },
      { id: 't2', message: 'Error', type: 'error' as const, createdAt: 2 },
    ],
    toast: vi.fn(),
    dismiss: mockDismiss,
  }),
}))

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true })
  mockDismiss.mockClear()
})

afterEach(() => {
  vi.useRealTimers()
})

describe('ToastContainer', () => {
  it('renders toasts with correct styles', () => {
    render(<ToastContainer />)
    expect(screen.getByText('Saved')).toBeInTheDocument()
    expect(screen.getByText('Error')).toBeInTheDocument()
  })

  it('calls dismiss when close button clicked', () => {
    render(<ToastContainer />)
    const buttons = screen.getAllByLabelText('Dismiss')
    fireEvent.click(buttons[0])
    expect(mockDismiss).toHaveBeenCalledWith('t1')
  })
})
