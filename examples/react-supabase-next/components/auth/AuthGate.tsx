'use client'

import { useEffect, useState, type ReactNode } from 'react'
import type { Session } from '@supabase/supabase-js'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

export default function AuthGate({ children }: { children: ReactNode }) {
  const [loading, setLoading] = useState(true)
  const [session, setSession] = useState<Session | null>(null)

  const configError = !process.env.NEXT_PUBLIC_SUPABASE_URL
    ? 'Missing NEXT_PUBLIC_SUPABASE_URL in .env.local'
    : !process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY
      ? 'Missing NEXT_PUBLIC_SUPABASE_ANON_KEY in .env.local'
      : null

  useEffect(() => {
    if (configError) {
      setLoading(false)
      return
    }

    const supabase = getSupabaseBrowserClient()
    supabase.auth.getSession().then(({ data }) => {
      setSession(data.session)
      setLoading(false)
      if (!data.session) window.location.replace('/login')
    })

    const { data: authListener } = supabase.auth.onAuthStateChange((_event, nextSession) => {
      setSession(nextSession)
      if (!nextSession) window.location.replace('/login')
    })

    return () => authListener.subscription.unsubscribe()
  }, [configError])

  if (configError) return <p style={{ color: '#b91c1c' }}>{configError}</p>
  if (loading) return <p>Checking auth session...</p>
  if (!session) return <p>Redirecting to login...</p>
  return <>{children}</>
}
