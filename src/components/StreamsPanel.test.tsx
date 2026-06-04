import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { StreamsPanel } from './StreamsPanel'

describe('StreamsPanel', () => {
  it('renders create form and empty state', () => {
    render(
      <StreamsPanel
        streams={[]}
        createTitle=""
        onCreateTitleChange={vi.fn()}
        createDesc=""
        onCreateDescChange={vi.fn()}
        error={null}
        onCreate={vi.fn()}
        onSelect={vi.fn()}
        onDelete={vi.fn()}
      />
    )
    expect(screen.getByText('Create New Stream')).toBeInTheDocument()
    expect(screen.getByText('No streams yet')).toBeInTheDocument()
  })

  it('calls onCreate when button clicked', () => {
    const onCreate = vi.fn()
    render(
      <StreamsPanel
        streams={[]}
        createTitle="My Stream"
        onCreateTitleChange={vi.fn()}
        createDesc=""
        onCreateDescChange={vi.fn()}
        error={null}
        onCreate={onCreate}
        onSelect={vi.fn()}
        onDelete={vi.fn()}
      />
    )
    fireEvent.click(screen.getByText('Create Stream'))
    expect(onCreate).toHaveBeenCalled()
  })

  it('calls onSelect when stream card clicked', () => {
    const onSelect = vi.fn()
    const streams = [
      {
        id: 's1',
        user_id: 'u1',
        title: 'AI Papers',
        description: '',
        is_public: true,
        created_at: '2024-01-01',
        updated_at: '2024-01-01',
      },
    ]
    render(
      <StreamsPanel
        streams={streams}
        createTitle=""
        onCreateTitleChange={vi.fn()}
        createDesc=""
        onCreateDescChange={vi.fn()}
        error={null}
        onCreate={vi.fn()}
        onSelect={onSelect}
        onDelete={vi.fn()}
      />
    )
    fireEvent.click(screen.getByText('AI Papers'))
    expect(onSelect).toHaveBeenCalledWith(streams[0])
  })

  it('calls onDelete when delete button clicked', () => {
    const onDelete = vi.fn()
    const streams = [
      {
        id: 's1',
        user_id: 'u1',
        title: 'AI Papers',
        description: '',
        is_public: true,
        created_at: '2024-01-01',
        updated_at: '2024-01-01',
      },
    ]
    render(
      <StreamsPanel
        streams={streams}
        createTitle=""
        onCreateTitleChange={vi.fn()}
        createDesc=""
        onCreateDescChange={vi.fn()}
        error={null}
        onCreate={vi.fn()}
        onSelect={vi.fn()}
        onDelete={onDelete}
      />
    )
    fireEvent.click(screen.getByText('Delete'))
    expect(onDelete).toHaveBeenCalledWith('s1')
  })
})
