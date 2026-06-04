import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { CapturePanel } from './CapturePanel'

describe('CapturePanel', () => {
  it('renders capture button', () => {
    render(
      <CapturePanel
        streamingSummary=""
        streamingTags={[]}
        streamingConnection={null}
        isStreaming={false}
        lastResult={null}
        error={null}
        onCapture={vi.fn()}
        modelStatus={null}
      />
    )
    expect(screen.getByText('Capture')).toBeInTheDocument()
  })

  it('disables button while streaming', () => {
    render(
      <CapturePanel
        streamingSummary="Loading..."
        streamingTags={[]}
        streamingConnection={null}
        isStreaming={true}
        lastResult={null}
        error={null}
        onCapture={vi.fn()}
        modelStatus={null}
      />
    )
    const btn = screen.getByRole('button')
    expect(btn).toBeDisabled()
    expect(btn).toHaveTextContent('Enriching')
  })

  it('displays error', () => {
    render(
      <CapturePanel
        streamingSummary=""
        streamingTags={[]}
        streamingConnection={null}
        isStreaming={false}
        lastResult={null}
        error="Clipboard is empty"
        onCapture={vi.fn()}
        modelStatus={null}
      />
    )
    expect(screen.getByText('Clipboard is empty')).toBeInTheDocument()
  })

  it('calls onCapture when clicked', () => {
    const onCapture = vi.fn()
    render(
      <CapturePanel
        streamingSummary=""
        streamingTags={[]}
        streamingConnection={null}
        isStreaming={false}
        lastResult={null}
        error={null}
        onCapture={onCapture}
        modelStatus={null}
      />
    )
    fireEvent.click(screen.getByText('Capture'))
    expect(onCapture).toHaveBeenCalled()
  })
})
