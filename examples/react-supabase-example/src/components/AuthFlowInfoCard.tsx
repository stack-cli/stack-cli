'use client'

import { useEffect, useMemo, useState } from 'react'
import type { Session } from '@supabase/supabase-js'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'
import {
  getLastAuthCall,
  subscribeFlowState,
  type LastAuthCall,
} from '@/lib/supabase/flowState'

type AuthFlowInfoCardProps = {
  mode: 'login' | 'signup'
}

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

export default function AuthFlowInfoCard({ mode }: AuthFlowInfoCardProps) {
  const [session, setSession] = useState<Session | null>(null)
  const [lastCall, setLastCall] = useState<LastAuthCall | null>(null)

  const configError = !import.meta.env.VITE_SUPABASE_URL
    ? 'Missing VITE_SUPABASE_URL in .env.local'
    : !import.meta.env.VITE_SUPABASE_ANON_KEY
      ? 'Missing VITE_SUPABASE_ANON_KEY in .env.local'
      : null

  useEffect(() => {
    setLastCall(getLastAuthCall())

    const unsubscribeFlow = subscribeFlowState(() => {
      setLastCall(getLastAuthCall())
    })

    if (configError) {
      return unsubscribeFlow
    }

    const supabase = getSupabaseBrowserClient()
    supabase.auth.getSession().then(({ data }) => {
      setSession(data.session)
    })

    const { data: authListener } = supabase.auth.onAuthStateChange(
      (_event, nextSession) => {
        setSession(nextSession)
      },
    )

    return () => {
      unsubscribeFlow()
      authListener.subscription.unsubscribe()
    }
  }, [configError])

  const expectedCalls = useMemo(() => {
    if (mode === 'login') {
      return ['POST /auth/v1/token?grant_type=password']
    }

    return [
      'POST /auth/v1/signup',
      'POST /auth/v1/token?grant_type=password (follow-up auto login in this demo)',
    ]
  }, [mode])

  const jwtPayload = useMemo(() => {
    return parseJwtPayload(session?.access_token ?? null)
  }, [session?.access_token])

  return (
    <section className="rounded-xl border bg-slate-50 p-4">
      <h2 className="text-base font-semibold">Auth API Flow</h2>
      <p className="mt-1 text-sm text-gray-700">
        What this page will call and what happened most recently.
      </p>

      <div className="mt-3 rounded border bg-white p-3">
        <p className="text-sm font-medium">Will happen on submit</p>
        <ul className="mt-2 space-y-1 font-mono text-xs">
          {expectedCalls.map(call => (
            <li key={call}>{call}</li>
          ))}
        </ul>
      </div>

      <div className="mt-3 rounded border bg-white p-3">
        <p className="text-sm font-medium">Last API call made</p>
        {!lastCall && (
          <p className="mt-2 font-mono text-xs text-gray-600">No auth API call recorded yet.</p>
        )}
        {lastCall && (
          <div className="mt-2 space-y-1 font-mono text-xs">
            <p>at={lastCall.at}</p>
            <p>request={lastCall.method} {lastCall.path}</p>
            <p>status={lastCall.status ?? '<none>'}</p>
            <p>ok={String(lastCall.ok)}</p>
            {lastCall.error && <p>error={lastCall.error}</p>}
          </div>
        )}
      </div>

      <div className="mt-3 rounded border bg-white p-3">
        <p className="text-sm font-medium">Session / JWT state</p>
        <div className="mt-2 space-y-1 font-mono text-xs">
          <p>base_url={import.meta.env.VITE_SUPABASE_URL ?? '<missing>'}</p>
          <p>anon_key={import.meta.env.VITE_SUPABASE_ANON_KEY ? '<present>' : '<missing>'}</p>
          <p>session={session ? 'present' : 'none'}</p>
          <p>jwt_retrieved={session?.access_token ? 'yes' : 'no'}</p>
          {jwtPayload && <p>jwt.role={String(jwtPayload.role ?? '<none>')}</p>}
          {jwtPayload && <p>jwt.aud={String(jwtPayload.aud ?? '<none>')}</p>}
          {jwtPayload && <p>jwt.exp={String(jwtPayload.exp ?? '<none>')}</p>}
        </div>
        {configError && <p className="mt-2 text-sm text-red-700">{configError}</p>}
      </div>
    </section>
  )
}

