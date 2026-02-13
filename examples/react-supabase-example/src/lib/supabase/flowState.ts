'use client'

export type LastAuthCall = {
  at: string
  method: string
  path: string
  status?: number
  ok: boolean
  error?: string
}

const LAST_CALL_KEY = 'auth.flow.last_call'
const UPDATE_EVENT = 'auth-flow-updated'

export function setLastAuthCall(call: LastAuthCall) {
  window.localStorage.setItem(LAST_CALL_KEY, JSON.stringify(call))
  window.dispatchEvent(new Event(UPDATE_EVENT))
}

export function getLastAuthCall(): LastAuthCall | null {
  const raw = window.localStorage.getItem(LAST_CALL_KEY)
  if (!raw) {
    return null
  }

  try {
    return JSON.parse(raw) as LastAuthCall
  } catch {
    return null
  }
}

export function subscribeFlowState(onChange: () => void) {
  const handler = () => onChange()
  window.addEventListener(UPDATE_EVENT, handler)
  window.addEventListener('storage', handler)

  return () => {
    window.removeEventListener(UPDATE_EVENT, handler)
    window.removeEventListener('storage', handler)
  }
}

