import { useState, useEffect, useCallback } from 'react'

export type ToastType = 'error' | 'success' | 'info'

export interface Toast {
  id: string
  message: string
  type: ToastType
  createdAt: number
  exiting?: boolean
}

interface Listener {
  setToasts: (toasts: Toast[]) => void
}

let globalToasts: Toast[] = []
const listeners = new Set<Listener>()
const timers: Record<string, number> = {}
const exitTimers: Record<string, number> = {}

const EXIT_DURATION = 300

function broadcast() {
  listeners.forEach((l) => l.setToasts([...globalToasts]))
}

function internalRemove(id: string) {
  globalToasts = globalToasts.filter((t) => t.id !== id)
  if (timers[id]) {
    window.clearTimeout(timers[id])
    delete timers[id]
  }
  if (exitTimers[id]) {
    window.clearTimeout(exitTimers[id])
    delete exitTimers[id]
  }
  broadcast()
}

function internalDismiss(id: string) {
  const toast = globalToasts.find((t) => t.id === id)
  if (!toast) return
  toast.exiting = true
  broadcast()
  exitTimers[id] = window.setTimeout(() => {
    internalRemove(id)
  }, EXIT_DURATION)
}

function internalShow(message: string, type: ToastType = 'info') {
  const id = crypto.randomUUID()
  const toast: Toast = { id, message, type, createdAt: Date.now() }
  globalToasts = [...globalToasts, toast]
  broadcast()
  const timer = window.setTimeout(() => {
    internalDismiss(id)
  }, 3000)
  timers[id] = timer
}

export interface UseToastReturn {
  toasts: Toast[]
  toast: (opts: { message: string; type?: ToastType }) => void
  dismiss: (id: string) => void
}

export function useToast(): UseToastReturn {
  const [localToasts, setLocalToasts] = useState<Toast[]>(() => [...globalToasts])

  useEffect(() => {
    const listener: Listener = { setToasts: setLocalToasts }
    listeners.add(listener)
    return () => {
      listeners.delete(listener)
    }
  }, [])

  const dismiss = useCallback((id: string) => {
    internalDismiss(id)
  }, [])

  const toast = useCallback((opts: { message: string; type?: ToastType }) => {
    internalShow(opts.message, opts.type ?? 'info')
  }, [])

  return { toasts: localToasts, toast, dismiss }
}

export function __resetToastsForTests() {
  globalToasts = []
  Object.keys(timers).forEach((id) => {
    window.clearTimeout(timers[id])
    delete timers[id]
  })
  Object.keys(exitTimers).forEach((id) => {
    window.clearTimeout(exitTimers[id])
    delete exitTimers[id]
  })
  listeners.clear()
}
