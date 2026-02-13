'use client'

import { FormEvent, useEffect, useState } from 'react'
import AuthGate from '@/components/auth/AuthGate'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'
import {
  insertRealtimeEntry,
  listRealtimeEntries,
  type RealtimeEntry,
} from '@/lib/supabase/api'

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
    const configError = !import.meta.env.VITE_SUPABASE_URL
      ? 'Missing VITE_SUPABASE_URL in .env.local'
      : !import.meta.env.VITE_SUPABASE_ANON_KEY
        ? 'Missing VITE_SUPABASE_ANON_KEY in .env.local'
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
      .on(
        'postgres_changes',
        { event: '*', schema: 'public', table: 'realtime_entries' },
        () => {
          void loadTopFive()
        },
      )
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
      <div className="space-y-6">
        <h1 className="text-3xl font-semibold">Realtime</h1>
        <p className="text-gray-700">
          Client-side realtime demo: the list subscribes to changes and updates without page refresh.
        </p>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">Last 5 Realtime Entries</h2>
          <p className="mt-2 font-mono text-xs text-gray-700">
            Calls:
            {' '}
            <code>supabase.from('realtime_entries').select(...).order(...).limit(5)</code>
            {' '}
            and
            {' '}
            <code>supabase.channel(...).on('postgres_changes', ...).subscribe()</code>
          </p>
          {loading && <p className="mt-2 text-sm text-gray-700">Loading entries...</p>}
          {!loading && (
            <div className="mt-3 overflow-auto">
              <table className="min-w-full border-collapse text-sm">
                <thead>
                  <tr>
                    <th className="border px-2 py-1 text-left">id</th>
                    <th className="border px-2 py-1 text-left">message</th>
                    <th className="border px-2 py-1 text-left">created_at</th>
                  </tr>
                </thead>
                <tbody>
                  {entries.length === 0 && (
                    <tr>
                      <td className="border px-2 py-1" colSpan={3}>No rows yet.</td>
                    </tr>
                  )}
                  {entries.map(entry => (
                    <tr key={entry.id}>
                      <td className="border px-2 py-1 font-mono text-xs">{entry.id}</td>
                      <td className="border px-2 py-1">{entry.message}</td>
                      <td className="border px-2 py-1 font-mono text-xs">{entry.created_at}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">Insert Realtime Entry</h2>
          <p className="mt-2 font-mono text-xs text-gray-700">
            Call:
            {' '}
            <code>supabase.from('realtime_entries').insert(...).select('id')</code>
          </p>
          <form onSubmit={onSubmit} className="mt-3 flex gap-2">
            <input
              type="text"
              value={message}
              onChange={event => setMessage(event.target.value)}
              placeholder="Enter message"
              className="w-full rounded border px-3 py-2"
              required
            />
            <button
              type="submit"
              disabled={submitting}
              className="rounded bg-black px-4 py-2 text-white disabled:opacity-60"
            >
              {submitting ? 'Saving...' : 'Save'}
            </button>
          </form>
        </section>

        {error && <p className="text-sm text-red-700">{error}</p>}
      </div>
    </AuthGate>
  )
}
