'use client'

import { useEffect, useState, type ReactNode } from 'react'
import type { Session } from '@supabase/supabase-js'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

type AuthGateProps = {
  children: ReactNode
}

export default function AuthGate({ children }: AuthGateProps) {
  const [loading, setLoading] = useState(true)
  const [session, setSession] = useState<Session | null>(null)

  const configError = !import.meta.env.VITE_SUPABASE_URL
    ? 'Missing VITE_SUPABASE_URL in .env.local'
    : !import.meta.env.VITE_SUPABASE_ANON_KEY
      ? 'Missing VITE_SUPABASE_ANON_KEY in .env.local'
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

      if (!data.session) {
        window.location.replace('/login')
      }
    })

    const { data: authListener } = supabase.auth.onAuthStateChange(
      (_event, nextSession) => {
        setSession(nextSession)
        if (!nextSession) {
          window.location.replace('/login')
        }
      },
    )

    return () => {
      authListener.subscription.unsubscribe()
    }
  }, [configError])

  if (configError) {
    return <p className="text-sm text-red-700">{configError}</p>
  }

  if (loading) {
    return <p className="text-sm text-gray-700">Checking auth session...</p>
  }

  if (!session) {
    return <p className="text-sm text-gray-700">Redirecting to login...</p>
  }

  return <>{children}</>
}

