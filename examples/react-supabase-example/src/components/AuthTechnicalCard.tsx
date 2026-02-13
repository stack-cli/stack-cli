'use client'

import { useEffect, useMemo, useState } from 'react'
import type { Session } from '@supabase/supabase-js'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'
import {
  addAuthEvent,
  clearAuthTraces,
  getAuthTraces,
  subscribeAuthTraces,
  type AuthTraceEvent,
} from '@/lib/supabase/authTrace'

function parseJwtPayload(token: string | null) {
  if (!token) {
    return null
  }

  const parts = token.split('.')
  if (parts.length < 2) {
    return null
  }

  try {
    const base64 = parts[1].replace(/-/g, '+').replace(/_/g, '/')
    const padded = base64.padEnd(Math.ceil(base64.length / 4) * 4, '=')
    const decoded = atob(padded)
    return JSON.parse(decoded) as Record<string, unknown>
  } catch {
    return null
  }
}

export default function AuthTechnicalCard() {
  const [session, setSession] = useState<Session | null>(null)
  const [events, setEvents] = useState<AuthTraceEvent[]>([])

  const configError = !import.meta.env.VITE_SUPABASE_URL
    ? 'Missing VITE_SUPABASE_URL in .env.local'
    : !import.meta.env.VITE_SUPABASE_ANON_KEY
      ? 'Missing VITE_SUPABASE_ANON_KEY in .env.local'
      : null

  useEffect(() => {
    setEvents(getAuthTraces())

    if (configError) {
      return
    }

    const supabase = getSupabaseBrowserClient()
    supabase.auth.getSession().then(({ data }) => {
      setSession(data.session)
    })

    const { data: authListener } = supabase.auth.onAuthStateChange(
      (event, nextSession) => {
        setSession(nextSession)
        addAuthEvent(`auth_state_change ${event}`)
        setEvents(getAuthTraces())
      },
    )

    const unsubscribeTrace = subscribeAuthTraces(() => {
      setEvents(getAuthTraces())
    })

    return () => {
      authListener.subscription.unsubscribe()
      unsubscribeTrace()
    }
  }, [configError])

  const jwtPayload = useMemo(() => {
    return parseJwtPayload(session?.access_token ?? null)
  }, [session?.access_token])

  return (
    <section className="rounded-xl border bg-slate-50 p-4">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-base font-semibold">Auth Technical Trace</h2>
        <button
          type="button"
          onClick={clearAuthTraces}
          className="rounded border px-2 py-1 text-xs"
        >
          Clear trace
        </button>
      </div>

      <div className="mt-3 space-y-1 font-mono text-xs">
        <p>base_url={import.meta.env.VITE_SUPABASE_URL ?? '<missing>'}</p>
        <p>anon_key={import.meta.env.VITE_SUPABASE_ANON_KEY ? '<present>' : '<missing>'}</p>
        <p>session={session ? 'present' : 'none'}</p>
      </div>

      {configError && (
        <p className="mt-2 text-sm text-red-700">setup_error={configError}</p>
      )}

      {jwtPayload && (
        <div className="mt-3 rounded border bg-white p-3">
          <p className="font-mono text-xs">jwt.role={String(jwtPayload.role ?? '<none>')}</p>
          <p className="font-mono text-xs">jwt.aud={String(jwtPayload.aud ?? '<none>')}</p>
          <p className="font-mono text-xs">jwt.exp={String(jwtPayload.exp ?? '<none>')}</p>
          <p className="font-mono text-xs">jwt.iss={String(jwtPayload.iss ?? '<none>')}</p>
        </div>
      )}

      <div className="mt-3 rounded border bg-white p-3">
        <p className="text-sm font-medium">Recent Auth Calls</p>
        {events.length === 0 && (
          <p className="mt-2 font-mono text-xs text-gray-600">No auth calls captured yet.</p>
        )}
        {events.length > 0 && (
          <ul className="mt-2 space-y-1 font-mono text-xs">
            {events.slice(0, 12).map((event, index) => (
              <li key={`${event.at}-${index}`} className="break-all">
                {event.type === 'http'
                  ? `${event.at} ${event.method ?? '-'} ${event.path ?? '-'} status=${event.status ?? '-'} ok=${String(event.ok)} dur=${event.durationMs ?? '-'}ms${event.message ? ` err="${event.message}"` : ''}`
                  : `${event.at} event ${event.message ?? ''}`}
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  )
}
