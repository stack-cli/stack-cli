'use client'

export type AuthTraceEvent = {
  at: string
  type: 'http' | 'event'
  method?: string
  path?: string
  status?: number
  durationMs?: number
  ok?: boolean
  message?: string
}

const STORAGE_KEY = 'auth.trace.events'
const MAX_EVENTS = 50
const UPDATE_EVENT = 'auth-trace-updated'

function readEvents(): AuthTraceEvent[] {
  const raw = window.localStorage.getItem(STORAGE_KEY)
  if (!raw) {
    return []
  }

  try {
    const parsed = JSON.parse(raw) as AuthTraceEvent[]
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

function writeEvents(events: AuthTraceEvent[]) {
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(events.slice(0, MAX_EVENTS)))
  window.dispatchEvent(new Event(UPDATE_EVENT))
}

export function addAuthTrace(event: AuthTraceEvent) {
  const existing = readEvents()
  writeEvents([event, ...existing])
}

export function addAuthEvent(message: string) {
  addAuthTrace({
    at: new Date().toISOString(),
    type: 'event',
    message,
  })
}

export function getAuthTraces(): AuthTraceEvent[] {
  return readEvents()
}

export function clearAuthTraces() {
  window.localStorage.removeItem(STORAGE_KEY)
  window.dispatchEvent(new Event(UPDATE_EVENT))
}

export function subscribeAuthTraces(onChange: () => void) {
  const handler = () => onChange()
  window.addEventListener(UPDATE_EVENT, handler)
  window.addEventListener('storage', handler)

  return () => {
    window.removeEventListener(UPDATE_EVENT, handler)
    window.removeEventListener('storage', handler)
  }
}

