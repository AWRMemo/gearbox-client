import { useToast } from '../hooks'
import type { ToastType } from '../hooks/useToast'

const borderClassMap: Record<ToastType, string> = {
  error: 'toast-border-error',
  success: 'toast-border-success',
  info: 'toast-border-info',
}

export function ToastContainer() {
  const { toasts, dismiss } = useToast()

  if (toasts.length === 0) return null

  return (
    <div
      className="toast-container"
      aria-live="polite"
      aria-atomic="false"
    >
      {toasts.map((t) => (
        <div
          key={t.id}
          role="alert"
          className={`toast-item ${borderClassMap[t.type]} ${t.exiting ? 'exiting' : ''}`}
        >
          <span className="toast-message">{t.message}</span>
          <button
            aria-label="Dismiss"
            className="toast-close"
            onClick={() => dismiss(t.id)}
          >
            ✕
          </button>
        </div>
      ))}
    </div>
  )
}
