import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { FollowingFeed } from './FollowingFeed'

vi.mock('../lib/tauri', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}))

beforeEach(() => {
  vi.clearAllMocks()
})

describe('FollowingFeed', () => {
  it('renders empty state when no subscriptions', async () => {
    render(<FollowingFeed />)
    expect(await screen.findByText('Not following any Streams')).toBeInTheDocument()
  })

  it('renders heading and refresh button after load', async () => {
    render(<FollowingFeed />)
    expect(screen.getByText('Following')).toBeInTheDocument()
    expect(await screen.findByText('Refresh')).toBeInTheDocument()
  })
})
