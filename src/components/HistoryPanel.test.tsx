import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { HistoryPanel } from './HistoryPanel'

vi.mock('../hooks/useHistory', () => ({ useHistory: vi.fn() }))
vi.mock('../hooks/useToast', () => ({ useToast: () => ({ toast: vi.fn() }) }))

import { useHistory } from '../hooks/useHistory'
const mockUseHistory = useHistory as ReturnType<typeof vi.fn>

describe('HistoryPanel', () => {
  const baseReturn = {
    entries: [],
    isLoading: false,
    hasMore: false,
    loadMore: vi.fn(),
    deleteHighlight: vi.fn(),
  }

  it('renders empty state', () => {
    mockUseHistory.mockReturnValue(baseReturn)
    render(<HistoryPanel />)
    expect(screen.getByText('No highlights yet')).toBeInTheDocument()
  })

  it('renders skeleton rows while loading with no data', () => {
    mockUseHistory.mockReturnValue({ ...baseReturn, isLoading: true })
    render(<HistoryPanel />)
    expect(screen.getByText('History')).toBeInTheDocument()
    expect(document.querySelectorAll('.skeleton-row').length).toBeGreaterThan(0)
  })

  it('renders highlights', () => {
    mockUseHistory.mockReturnValue({
      ...baseReturn,
      entries: [
        {
          id: 'h1',
          text: 't1',
          source_url: 'https://example.com',
          source_title: 'Example',
          source_author: null,
          summary: 'Summary one',
          tags: ['rust'],
          created_at: '2024-01-01',
        },
      ],
    })
    render(<HistoryPanel />)
    expect(screen.getByText('Summary one')).toBeInTheDocument()
    expect(screen.getByText('rust')).toBeInTheDocument()
    expect(screen.getByText('Delete')).toBeInTheDocument()
  })

  it('calls loadMore when button clicked', () => {
    const loadMore = vi.fn()
    mockUseHistory.mockReturnValue({
      ...baseReturn,
      entries: [
        {
          id: 'h1',
          text: 't1',
          source_url: null,
          source_title: null,
          source_author: null,
          summary: 'S',
          tags: [],
          created_at: '2024-01-01',
        },
      ],
      hasMore: true,
      loadMore,
    })
    render(<HistoryPanel />)
    fireEvent.click(screen.getByText('Load more'))
    expect(loadMore).toHaveBeenCalled()
  })

  it('calls deleteHighlight with confirmation', () => {
    const deleteHighlight = vi.fn()
    vi.stubGlobal('confirm', vi.fn(() => true))
    mockUseHistory.mockReturnValue({
      ...baseReturn,
      entries: [
        {
          id: 'h1',
          text: 't1',
          source_url: null,
          source_title: null,
          source_author: null,
          summary: 'S',
          tags: [],
          created_at: '2024-01-01',
        },
      ],
      deleteHighlight,
    })
    render(<HistoryPanel />)
    fireEvent.click(screen.getByText('Delete'))
    expect(deleteHighlight).toHaveBeenCalledWith('h1')
    vi.unstubAllGlobals()
  })
})
