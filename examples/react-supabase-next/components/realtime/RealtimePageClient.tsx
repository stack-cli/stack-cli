'use client'

import { FormEvent, useEffect, useState } from 'react'
import AuthGate from '@/components/auth/AuthGate'
import {
  insertRealtimeEntry,
  listRealtimeEntries,
  type RealtimeEntry,
} from '@/lib/supabase/api'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

export default function RealtimePageClient() {
  const [loading, setLoading] = useState(true)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState('')
  const [entries, setEntries] = useState<RealtimeEntry[]>([])

  async function loadTopFive() {
    const supabase = getSupabaseBrowserClient()
    const { items, ok, result } = await listRealtimeEntries(supabase, 5)
    if (!ok) {
      setError(result)
      return
    }
    setEntries(items)
  }

  useEffect(() => {
    const configError = !process.env.NEXT_PUBLIC_SUPABASE_URL
      ? 'Missing NEXT_PUBLIC_SUPABASE_URL in .env.local'
      : !process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY
        ? 'Missing NEXT_PUBLIC_SUPABASE_ANON_KEY in .env.local'
        : null

    if (configError) {
      setError(configError)
      setLoading(false)
      return
    }

    const supabase = getSupabaseBrowserClient()
    const run = async () => {
      const { data } = await supabase.auth.getSession()
      if (!data.session) {
        setError('No authenticated session')
        setLoading(false)
        return
      }
      await loadTopFive()
      setLoading(false)
    }

    const channel = supabase
      .channel('realtime-entries-feed')
      .on('postgres_changes', { event: '*', schema: 'public', table: 'realtime_entries' }, () => {
        void loadTopFive()
      })
      .subscribe()

    run().catch((nextError: unknown) => {
      setError(nextError instanceof Error ? nextError.message : 'Realtime page failed')
      setLoading(false)
    })

    return () => {
      void supabase.removeChannel(channel)
    }
  }, [])

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    setSubmitting(true)

    const supabase = getSupabaseBrowserClient()
    const { data } = await supabase.auth.getSession()

    if (!data.session?.user?.id) {
      setError('No authenticated session')
      setSubmitting(false)
      return
    }

    const { ok, result } = await insertRealtimeEntry(supabase, message, data.session.user.id)
    if (!ok) {
      setError(result)
      setSubmitting(false)
      return
    }

    setMessage('')
    await loadTopFive()
    setSubmitting(false)
  }

  return (
    <AuthGate>
      <div className="stack">
        <h1 style={{ margin: 0 }}>Realtime</h1>
        <p className="muted">Client-side list + insert with realtime subscription updates.</p>

        <section className="card">
          <div className="row">
            <h2 style={{ margin: 0 }}>Last 5 Realtime Entries</h2>
            <span className="badge badge-client">Client</span>
          </div>
          <p className="muted" style={{ marginTop: '0.5rem' }}>React client component subscribing to DB change events.</p>
          <p className="mono" style={{ fontSize: '0.75rem' }}>
            Calls: supabase.from('realtime_entries').select(...).order(...).limit(5) and supabase.channel(...).on('postgres_changes', ...).subscribe()
          </p>
          {loading && <p className="muted">Loading entries...</p>}
          {!loading && (
            <div className="table-wrap">
              <table className="table">
                <thead>
                  <tr>
                    <th>id</th>
                    <th>message</th>
                    <th>created_at</th>
                  </tr>
                </thead>
                <tbody>
                  {entries.length === 0 && (
                    <tr>
                      <td colSpan={3}>No rows yet.</td>
                    </tr>
                  )}
                  {entries.map((entry) => (
                    <tr key={entry.id}>
                      <td className="mono" style={{ fontSize: '0.75rem' }}>{entry.id}</td>
                      <td>{entry.message}</td>
                      <td className="mono" style={{ fontSize: '0.75rem' }}>{entry.created_at}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>

        <section className="card">
          <div className="row">
            <h2 style={{ margin: 0 }}>Insert Realtime Entry</h2>
            <span className="badge badge-client">Client</span>
          </div>
          <p className="muted" style={{ marginTop: '0.5rem' }}>React client component writing directly via Supabase JS.</p>
          <p className="mono" style={{ fontSize: '0.75rem' }}>
            Calls: supabase.from('realtime_entries').insert(...).select('id')
          </p>
          <form onSubmit={onSubmit} className="row" style={{ marginTop: '0.75rem' }}>
            <input
              type="text"
              value={message}
              onChange={(event) => setMessage(event.target.value)}
              placeholder="Enter message"
              required
              className="input"
            />
            <button type="submit" disabled={submitting} className="btn">
              {submitting ? 'Saving...' : 'Save'}
            </button>
          </form>
        </section>

        {error && <p className="error">{error}</p>}
      </div>
    </AuthGate>
  )
}
