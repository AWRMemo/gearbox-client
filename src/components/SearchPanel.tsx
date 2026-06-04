import { EmptyState } from './EmptyState'
import { SkeletonRow } from './SkeletonRow'
import { SearchResultCard } from './SearchResultCard'
import type { SearchResult, SearchFilter, StreamInfo } from '../types'

interface SearchPanelProps {
  query: string
  onQueryChange: (q: string) => void
  results: SearchResult[]
  isSearching: boolean
  error: string | null
  selectedIndex: number
  selectedStream: StreamInfo | null
  onSearch: () => void
  onKeyDown: (e: React.KeyboardEvent) => void
  onAddToStream: (id: string) => void
  onSelect?: (id: string) => void
  semantic: boolean
  onSemanticChange: (v: boolean) => void
  filters: SearchFilter
  onFiltersChange: (f: SearchFilter) => void
  inputRef?: React.Ref<HTMLInputElement>
}

export function SearchPanel({
  query,
  onQueryChange,
  results,
  isSearching,
  error,
  selectedIndex,
  selectedStream,
  onSearch,
  onKeyDown,
  onAddToStream,
  onSelect,
  semantic,
  onSemanticChange,
  filters,
  onFiltersChange,
  inputRef,
}: SearchPanelProps) {
  return (
    <div style={{ marginTop: '1.5rem' }}>
      <h2>Search</h2>
      <div className="controls" style={{ marginTop: '0.5rem' }}>
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder="Search…"
          className="search-input"
          aria-label="Search query"
        />
        <button onClick={onSearch} disabled={isSearching}>
          {isSearching ? '…' : 'Search'}
        </button>
      </div>
      <div className="search-filters" style={{ marginTop: '0.5rem', display: 'flex', gap: '0.5rem', flexWrap: 'wrap', alignItems: 'center' }}>
        <label className="filter-label" style={{ display: 'flex', alignItems: 'center', gap: '0.3rem', fontSize: '0.85rem' }}>
          <input
            type="checkbox"
            checked={semantic}
            onChange={(e) => onSemanticChange(e.target.checked)}
          />
          Semantic
        </label>
        <input
          type="date"
          value={filters.dateFrom || ''}
          onChange={(e) => onFiltersChange({ ...filters, dateFrom: e.target.value || undefined })}
          className="search-filter-input"
          aria-label="Date from"
        />
        <input
          type="date"
          value={filters.dateTo || ''}
          onChange={(e) => onFiltersChange({ ...filters, dateTo: e.target.value || undefined })}
          className="search-filter-input"
          aria-label="Date to"
        />
        <input
          type="text"
          value={filters.sourceDomain || ''}
          onChange={(e) => onFiltersChange({ ...filters, sourceDomain: e.target.value || undefined })}
          placeholder="Domain filter"
          className="search-filter-input"
          aria-label="Filter by domain"
          style={{ width: '120px' }}
        />
      </div>
      {error && (
        <div className="result-card" style={{ borderLeft: '3px solid var(--danger-text)' }}>
          {error}
        </div>
      )}
      {isSearching && (
        <div style={{ marginTop: '0.5rem' }}>
          <SkeletonRow />
          <SkeletonRow />
          <SkeletonRow />
        </div>
      )}
      {results.length > 0 && !isSearching && (
        <div style={{ marginTop: '0.5rem' }}>
          <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: '0.5rem' }}>
            {results.length} result{results.length !== 1 ? 's' : ''}
            {filters.dateFrom || filters.dateTo || filters.sourceDomain ? ' (filtered)' : ''}
          </p>
          {results.map((r, i) => (
            <SearchResultCard
              key={r.id}
              result={r}
              index={i}
              selectedIndex={selectedIndex}
              onAddToStream={onAddToStream}
              selectedStream={selectedStream ? { title: selectedStream.title } : null}
              onSelect={onSelect}
            />
          ))}
        </div>
      )}
      {results.length === 0 && query && !isSearching && !error && (
        <div style={{ marginTop: '0.5rem' }}>
          <EmptyState
            title="No matches"
            description="Try different keywords or adjust filters."
          />
        </div>
      )}
      {results.length === 0 && !query && !isSearching && !error && (
        <EmptyState
          title="Search your knowledge base"
          description="Search by keyword or concept to find highlights."
        />
      )}
    </div>
  )
}

export type { SearchFilter }
