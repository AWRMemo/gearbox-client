import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { SearchPanel } from './SearchPanel'

function defaultProps(overrides: Record<string, unknown> = {}) {
  return {
    query: '',
    onQueryChange: vi.fn(),
    results: [],
    isSearching: false,
    error: null,
    selectedIndex: -1,
    selectedStream: null,
    onSearch: vi.fn(),
    onKeyDown: vi.fn(),
    onAddToStream: vi.fn(),
    semantic: false,
    onSemanticChange: vi.fn(),
    filters: {},
    onFiltersChange: vi.fn(),
    ...overrides,
  }
}

describe('SearchPanel', () => {
  it('renders input and button', () => {
    render(<SearchPanel {...defaultProps()} />)
    expect(screen.getByPlaceholderText('Search…')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /search/i })).toBeInTheDocument()
  })

  it('calls onSearch when button clicked', () => {
    const onSearch = vi.fn()
    render(<SearchPanel {...defaultProps({ query: 'fox', onSearch })} />)
    fireEvent.click(screen.getByRole('button', { name: /search/i }))
    expect(onSearch).toHaveBeenCalled()
  })

  it('calls onQueryChange on input change', () => {
    const onQueryChange = vi.fn()
    render(<SearchPanel {...defaultProps({ onQueryChange })} />)
    fireEvent.change(screen.getByPlaceholderText('Search…'), {
      target: { value: 'rust' },
    })
    expect(onQueryChange).toHaveBeenCalledWith('rust')
  })

  it('renders search results', () => {
    render(
      <SearchPanel
        {...defaultProps({
          query: 'fox',
          results: [
            {
              id: '1',
              summary: 'A fox',
              tags: ['fox'],
              text: 'The quick brown fox.',
              score: 0.9,
            },
          ],
        })}
      />
    )
    expect(screen.getByText('A fox')).toBeInTheDocument()
    expect(screen.getByText('fox')).toBeInTheDocument()
  })

  it('shows no results message', () => {
    render(<SearchPanel {...defaultProps({ query: 'zzz' })} />)
    expect(screen.getByText('No matches')).toBeInTheDocument()
    expect(screen.getByText('Try different keywords or adjust filters.')).toBeInTheDocument()
  })

  it('renders filter controls', () => {
    render(<SearchPanel {...defaultProps()} />)
    expect(screen.getByLabelText('Semantic')).toBeInTheDocument()
    expect(screen.getByLabelText('Date from')).toBeInTheDocument()
    expect(screen.getByLabelText('Date to')).toBeInTheDocument()
    expect(screen.getByLabelText('Filter by domain')).toBeInTheDocument()
  })

  it('calls onSemanticChange when checkbox toggled', () => {
    const onSemanticChange = vi.fn()
    render(<SearchPanel {...defaultProps({ onSemanticChange })} />)
    fireEvent.click(screen.getByLabelText('Semantic'))
    expect(onSemanticChange).toHaveBeenCalledWith(true)
  })

  it('calls onFiltersChange when domain filter changes', () => {
    const onFiltersChange = vi.fn()
    render(<SearchPanel {...defaultProps({ onFiltersChange })} />)
    fireEvent.change(screen.getByLabelText('Filter by domain'), {
      target: { value: 'example.com' },
    })
    expect(onFiltersChange).toHaveBeenCalledWith({
      sourceDomain: 'example.com',
    })
  })

  it('shows result count when results exist', () => {
    render(
      <SearchPanel
        {...defaultProps({
          query: 'fox',
          results: [
            { id: '1', summary: 'A fox', tags: ['fox'], text: 'text', score: 0.9 },
            { id: '2', summary: 'A wolf', tags: ['wolf'], text: 'text', score: 0.7 },
          ],
        })}
      />
    )
    expect(screen.getByText('2 results')).toBeInTheDocument()
  })
})
