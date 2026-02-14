'use client'

import Link from 'next/link'
import { useEffect, useState } from 'react'
import type { Session } from '@supabase/supabase-js'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'
import { clearSessionCookies, syncSessionCookies } from '@/lib/supabase/sessionCookie'

export default function AppNav() {
  const [session, setSession] = useState<Session | null>(null)
  const [loading, setLoading] = useState(true)

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
      syncSessionCookies(data.session)
      setLoading(false)
    })

    const { data: authListener } = supabase.auth.onAuthStateChange((_event, nextSession) => {
      setSession(nextSession)
      syncSessionCookies(nextSession)
    })

    return () => authListener.subscription.unsubscribe()
  }, [configError])

  async function signOut() {
    if (configError) return
    const supabase = getSupabaseBrowserClient()
    await supabase.auth.signOut()
    clearSessionCookies()
    window.location.replace('/login')
  }

  if (loading) return <nav style={{ borderBottom: '1px solid #e5e7eb', paddingBottom: 12 }}>Loading menu...</nav>
  if (configError) return <nav style={{ borderBottom: '1px solid #e5e7eb', paddingBottom: 12, color: '#b91c1c' }}>{configError}</nav>

  if (!session) {
    return (
      <nav style={{ display: 'flex', gap: 16, borderBottom: '1px solid #e5e7eb', paddingBottom: 12 }}>
        <Link href="/signup">Sign up</Link>
        <Link href="/login">Login</Link>
      </nav>
    )
  }

  return (
    <nav style={{ display: 'flex', gap: 16, borderBottom: '1px solid #e5e7eb', paddingBottom: 12, alignItems: 'center' }}>
      <Link href="/">Home</Link>
      <Link href="/realtime">Realtime</Link>
      <Link href="/realtime-broadcast">Realtime Broadcast</Link>
      <Link href="/postgrest">Postgrest</Link>
      <Link href="/storage">Storage</Link>
      <span style={{ marginLeft: 'auto', fontSize: 14, color: '#374151' }}>{session.user.email}</span>
      <button type="button" onClick={signOut}>Logout</button>
    </nav>
  )
}
