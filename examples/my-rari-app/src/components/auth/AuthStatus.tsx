'use client'

import { useEffect, useState } from 'react'
import type { Session } from '@supabase/supabase-js'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

export default function AuthStatus() {
  const [session, setSession] = useState<Session | null>(null)
  const [loading, setLoading] = useState(true)
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
    })

    const { data: authListener } = supabase.auth.onAuthStateChange(
      (_event, nextSession) => {
        setSession(nextSession)
      },
    )

    return () => {
      authListener.subscription.unsubscribe()
    }
  }, [configError])

  async function signOut() {
    const supabase = getSupabaseBrowserClient()
    await supabase.auth.signOut()
  }

  if (loading) {
    return <span className="text-sm text-gray-600">Checking auth...</span>
  }

  if (configError) {
    return <span className="text-sm text-red-700">{configError}</span>
  }

  if (!session) {
    return <span className="text-sm text-gray-600">Signed out</span>
  }

  return (
    <div className="flex items-center gap-3">
      <span className="text-sm text-gray-700">{session.user.email}</span>
      <button
        type="button"
        onClick={signOut}
        className="rounded border px-2 py-1 text-sm"
      >
        Logout
      </button>
    </div>
  )
}
