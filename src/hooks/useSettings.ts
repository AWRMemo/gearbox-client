import { useState, useCallback } from 'react'
import { invoke } from '../lib/tauri'
import { useToast } from './useToast'
import type { UserProfile } from '../types'

export interface UseSettingsReturn {
  profile: UserProfile | null
  emailInput: string
  setEmailInput: (e: string) => void
  message: string | null
  loadProfile: () => Promise<void>
  saveEmail: () => Promise<void>
  autoCapture: boolean
  setAutoCapture: (v: boolean) => Promise<void>
  telemetryEnabled: boolean
  setTelemetryEnabled: (v: boolean) => Promise<void>
}

export function useSettings(): UseSettingsReturn {
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [emailInput, setEmailInput] = useState('')
  const [message, setMessage] = useState<string | null>(null)
  const [autoCapture, setAutoCaptureLocal] = useState(true)
  const [telemetryEnabled, setTelemetryEnabledLocal] = useState(false)
  const { toast } = useToast()

  const loadProfile = useCallback(async () => {
    try {
      const p: UserProfile = await invoke('get_user_profile')
      setProfile(p)
      setEmailInput(p.email || '')
    } catch {
      // not critical
    }
    try {
      const enabled: boolean = await invoke('get_auto_capture_enabled')
      setAutoCaptureLocal(enabled)
    } catch {
      // not critical
    }
    try {
      const optOut: boolean = await invoke('get_telemetry_opt_out')
      setTelemetryEnabledLocal(!optOut)
    } catch {
      // not critical
    }
  }, [])

  const saveEmail = useCallback(async () => {
    try {
      await invoke('set_user_email', { email: emailInput.trim() })
      setMessage('Email saved.')
      setTimeout(() => setMessage(null), 2000)
    } catch (err) {
      setMessage(String(err))
    }
  }, [emailInput])

  const setAutoCapture = useCallback(async (v: boolean) => {
    try {
      await invoke('set_auto_capture_enabled', { enabled: v })
      setAutoCaptureLocal(v)
      toast({ message: v ? 'Auto-capture enabled' : 'Auto-capture disabled', type: 'info' })
    } catch (err) {
      toast({ message: `Failed to update auto-capture: ${String(err)}`, type: 'error' })
    }
  }, [toast])

  const setTelemetryEnabled = useCallback(async (v: boolean) => {
    try {
      await invoke('toggle_telemetry', { enabled: v })
      setTelemetryEnabledLocal(v)
      toast({ message: v ? 'Telemetry enabled' : 'Telemetry disabled', type: 'info' })
    } catch (err) {
      toast({ message: `Failed to update telemetry: ${String(err)}`, type: 'error' })
    }
  }, [toast])

  return { profile, emailInput, setEmailInput, message, loadProfile, saveEmail, autoCapture, setAutoCapture, telemetryEnabled, setTelemetryEnabled }
}
