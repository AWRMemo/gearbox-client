import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { TabBar } from './TabBar'

describe('TabBar', () => {
  it('renders all tabs', () => {
    render(
      <TabBar
        active="capture"
        onTabChange={vi.fn()}
        onLoadStreams={vi.fn()}
        onLoadFeed={vi.fn()}
        onLoadProfile={vi.fn()}
      />
    )
    expect(screen.getByText('Capture')).toBeInTheDocument()
    expect(screen.getByText('Feed')).toBeInTheDocument()
    expect(screen.getByText('Streams')).toBeInTheDocument()
    expect(screen.getByText('Settings')).toBeInTheDocument()
  })

  it('calls onTabChange when clicked', () => {
    const onChange = vi.fn()
    render(
      <TabBar
        active="capture"
        onTabChange={onChange}
        onLoadStreams={vi.fn()}
        onLoadFeed={vi.fn()}
        onLoadProfile={vi.fn()}
      />
    )
    fireEvent.click(screen.getByText('Streams'))
    expect(onChange).toHaveBeenCalledWith('streams')
  })

  it('marks active tab', () => {
    render(
      <TabBar
        active="feed"
        onTabChange={vi.fn()}
        onLoadStreams={vi.fn()}
        onLoadFeed={vi.fn()}
        onLoadProfile={vi.fn()}
      />
    )
    const feedBtn = screen.getByText('Feed')
    expect(feedBtn.className).toContain('tab-active')
  })
})
