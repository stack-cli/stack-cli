'use client'

import { useEffect, useState } from 'react'
import type { Session } from '@supabase/supabase-js'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

export default function AppNav() {
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
    if (configError) {
      return
    }

    const supabase = getSupabaseBrowserClient()
    await supabase.auth.signOut()
    window.location.replace('/login')
  }

  if (loading) {
    return (
      <nav className="flex flex-wrap items-center gap-4 border-b pb-4">
        <span className="text-sm text-gray-600">Loading menu...</span>
      </nav>
    )
  }

  if (configError) {
    return (
      <nav className="flex flex-wrap items-center gap-4 border-b pb-4">
        <span className="text-sm text-red-700">{configError}</span>
      </nav>
    )
  }

  if (!session) {
    return (
      <nav className="flex flex-wrap items-center gap-4 border-b pb-4">
        <a href="/signup">Sign up</a>
        <a href="/login">Login</a>
      </nav>
    )
  }

  return (
    <nav className="flex flex-wrap items-center gap-4 border-b pb-4">
      <a href="/">Home</a>
      <a href="/realtime">Realtime</a>
      <a href="/postgrest">Postgrest</a>
      <span className="ml-auto text-sm text-gray-700">{session.user.email}</span>
      <button type="button" onClick={signOut} className="rounded border px-2 py-1 text-sm">
        Logout
      </button>
    </nav>
  )
}
