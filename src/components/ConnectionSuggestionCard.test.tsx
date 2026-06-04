import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ConnectionSuggestionCard } from './ConnectionSuggestionCard'

beforeEach(() => {
  localStorage.clear()
})

describe('ConnectionSuggestionCard', () => {
  it('renders suggestion from JSON string', () => {
    const suggestion = JSON.stringify({
      source_highlight_id: 'src-123',
      bridging_sentence: 'Related to climate research',
      target_summary: 'How CO2 affects temperature',
    })

    render(
      <ConnectionSuggestionCard
        suggestion={suggestion}
        highlightId="h-1"
      />
    )

    expect(screen.getByText('Connection Suggestion')).toBeInTheDocument()
    expect(screen.getByText('Related to climate research')).toBeInTheDocument()
    expect(screen.getByText('How CO2 affects temperature')).toBeInTheDocument()
  })

  it('shows dismiss button and hides on click', () => {
    const suggestion = JSON.stringify({
      source_highlight_id: 'src-123',
      bridging_sentence: 'Test bridge',
    })

    render(
      <ConnectionSuggestionCard
        suggestion={suggestion}
        highlightId="h-2"
      />
    )

    const dismissBtn = screen.getByText('Dismiss')
    expect(dismissBtn).toBeInTheDocument()

    fireEvent.click(dismissBtn)
    expect(screen.queryByText('Connection Suggestion')).not.toBeInTheDocument()
  })

  it('renders nothing when suggestion is empty', () => {
    const { container } = render(
      <ConnectionSuggestionCard
        suggestion=""
        highlightId="h-3"
      />
    )
    expect(container.innerHTML).toBe('')
  })
})
