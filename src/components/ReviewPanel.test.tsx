import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ReviewPanel } from './ReviewPanel'

vi.mock('../lib/tauri', () => ({
  invoke: vi.fn().mockResolvedValue({ items: [], total_due: 0 }),
}))

vi.mock('../hooks/useToast', () => ({
  useToast: () => ({ toast: vi.fn() }),
}))

describe('ReviewPanel', () => {
  it('renders empty state when no session', async () => {
    render(<ReviewPanel />)
    expect(await screen.findByText('All caught up!')).toBeInTheDocument()
  })

  it('renders start review button', () => {
    render(<ReviewPanel />)
    expect(screen.getByText('Start Review')).toBeInTheDocument()
  })

  it('renders review heading', () => {
    render(<ReviewPanel />)
    expect(screen.getByText('Review')).toBeInTheDocument()
  })
})
