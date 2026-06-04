import { useState, useEffect } from 'react'
import { invoke } from '../lib/tauri'
import { useToast } from './useToast'
import type { AuthStatus } from '../types'

export interface AuthState {
  isAuthenticated: boolean
  email: string | null
  serverUrl: string | null
  loading: boolean
  error: string | null
}

const defaultState: AuthState = {
  isAuthenticated: false,
  email: null,
  serverUrl: null,
  loading: false,
  error: null,
}

let globalAuth: AuthState = { ...defaultState }
const listeners = new Set<(state: AuthState) => void>()

function broadcast() {
  listeners.forEach((l) => l({ ...globalAuth }))
}

export function __resetAuthForTests() {
  globalAuth = { ...defaultState }
  listeners.clear()
}

async function doCheckStatus() {
  try {
    const status: AuthStatus = await invoke('get_auth_status')
    globalAuth = {
      ...globalAuth,
      isAuthenticated: status.signed_in,
      email: status.email,
      serverUrl: status.server_url,
      error: null,
    }
    broadcast()
  } catch {
    globalAuth = { ...globalAuth, isAuthenticated: false, error: null }
    broadcast()
  }
}

export interface UseAuthReturn extends AuthState {
  createAccount: (email: string, password: string) => Promise<void>
  logIn: (email: string, password: string) => Promise<void>
  logOut: () => Promise<void>
}

export function useAuth(): UseAuthReturn {
  const { toast } = useToast()
  const [localState, setLocalState] = useState<AuthState>({ ...globalAuth })

  useEffect(() => {
    listeners.add(setLocalState)
    return () => {
      listeners.delete(setLocalState)
    }
  }, [])

  const createAccount = async (emailValue: string, password: string) => {
    globalAuth = { ...globalAuth, loading: true, error: null }
    broadcast()
    try {
      await invoke('create_account', { email: emailValue, password })
      globalAuth = { ...globalAuth, isAuthenticated: true, email: emailValue, loading: false, error: null }
      broadcast()
      toast({ message: 'Account created successfully', type: 'success' })
    } catch (err) {
      const msg = `Account creation failed: ${String(err)}`
      globalAuth = { ...globalAuth, loading: false, error: msg }
      broadcast()
      toast({ message: msg, type: 'error' })
    }
  }

  const logIn = async (emailValue: string, password: string) => {
    globalAuth = { ...globalAuth, loading: true, error: null }
    broadcast()
    try {
      await invoke('log_in', { email: emailValue, password })
      globalAuth = { ...globalAuth, isAuthenticated: true, email: emailValue, loading: false, error: null }
      broadcast()
      toast({ message: 'Signed in successfully', type: 'success' })
    } catch (err) {
      const msg = `Sign in failed: ${String(err)}`
      globalAuth = { ...globalAuth, loading: false, error: msg }
      broadcast()
      toast({ message: msg, type: 'error' })
    }
  }

  const logOut = async () => {
    globalAuth = { ...globalAuth, loading: true }
    broadcast()
    try {
      await invoke('log_out')
      globalAuth = { ...globalAuth, isAuthenticated: false, email: null, serverUrl: null, loading: false }
      broadcast()
      toast({ message: 'Signed out', type: 'info' })
    } catch (err) {
      const msg = `Sign out failed: ${String(err)}`
      globalAuth = { ...globalAuth, loading: false, error: msg }
      broadcast()
      toast({ message: msg, type: 'error' })
    }
  }

  // Initial check on mount via microtask to avoid setState-in-effect lint
  useEffect(() => {
    queueMicrotask(() => doCheckStatus())
  }, [])

  return {
    ...localState,
    createAccount,
    logIn,
    logOut,
  }
}
