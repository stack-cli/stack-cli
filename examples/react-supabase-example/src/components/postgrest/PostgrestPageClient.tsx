'use client'

import { FormEvent, useEffect, useState } from 'react'
import AuthGate from '@/components/auth/AuthGate'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'
import { insertDemoItem, listDemoItems, type DemoItem } from '@/lib/supabase/api'

type CallResult = {
  at: string
  call: string
  status: number | null
  ok: boolean
  result: string
}

export default function PostgrestPageClient() {
  const [loading, setLoading] = useState(true)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [title, setTitle] = useState('')
  const [items, setItems] = useState<DemoItem[]>([])
  const [calls, setCalls] = useState<CallResult[]>([])

  function recordCall(call: CallResult) {
    setCalls(current => [call, ...current].slice(0, 10))
  }

  async function loadTopFive() {
    const call = "supabase.from('demo_items').select(...).order('created_at', desc).limit(5)"

    try {
      const supabase = getSupabaseBrowserClient()
      const { items: nextItems, ok, result } = await listDemoItems(supabase, 5)
      setItems(nextItems)

      recordCall({
        at: new Date().toISOString(),
        call,
        status: null,
        ok,
        result,
      })

      if (!ok) {
        setError(result)
      }
    } catch (error) {
      recordCall({
        at: new Date().toISOString(),
        call,
        status: null,
        ok: false,
        result: error instanceof Error ? error.message : 'Request failed',
      })
      setError(error instanceof Error ? error.message : 'List request failed')
    }
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

    const run = async () => {
      const supabase = getSupabaseBrowserClient()
      const { data } = await supabase.auth.getSession()

      if (!data.session?.access_token) {
        setError('No access token in session')
        setLoading(false)
        return
      }

      await loadTopFive()
      setLoading(false)
    }

    run().catch((nextError: unknown) => {
      setError(nextError instanceof Error ? nextError.message : 'PostgREST call failed')
      setLoading(false)
    })
  }, [])

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    setSubmitting(true)

    const supabase = getSupabaseBrowserClient()
    const { data } = await supabase.auth.getSession()

    if (!data.session?.access_token || !data.session.user?.id) {
      setSubmitting(false)
      setError('No authenticated session')
      return
    }

    const call = "supabase.from('demo_items').insert({ title, user_id }).select('id')"

    try {
      const supabase = getSupabaseBrowserClient()
      const { ok, result } = await insertDemoItem(supabase, title, data.session.user.id)

      recordCall({
        at: new Date().toISOString(),
        call,
        status: null,
        ok,
        result,
      })

      if (!ok) {
        setSubmitting(false)
        setError(result)
        return
      }

      setTitle('')
      await loadTopFive()
    } catch (nextError) {
      recordCall({
        at: new Date().toISOString(),
        call,
        status: null,
        ok: false,
        result: nextError instanceof Error ? nextError.message : 'Request failed',
      })
      setError(nextError instanceof Error ? nextError.message : 'Insert request failed')
    }

    setSubmitting(false)
  }

  return (
    <AuthGate>
      <div className="space-y-6">
        <h1 className="text-3xl font-semibold">PostgREST</h1>
        <p className="text-gray-700">
          This page uses PostgREST from a client component. It reads the top 5 rows
          and inserts new rows directly through
          {' '}
          <code>/rest/v1/demo_items</code>
          {' '}
          using your access token.
        </p>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">Top 5 Rows (client-side GET)</h2>
          <p className="mt-2 text-xs text-gray-700">
            Runtime:
            {' '}
            <strong>React Client Component</strong>
            {' '}
            (runs in the browser after hydration).
          </p>
          <p className="mt-1 font-mono text-xs text-gray-700">
            Call:
            {' '}
            <code>supabase.from('demo_items').select('id,title,created_at').order('created_at', desc).limit(5)</code>
          </p>
          {loading && <p className="mt-2 text-sm text-gray-700">Loading rows...</p>}
          {!loading && (
            <div className="mt-3 overflow-auto">
              <table className="min-w-full border-collapse text-sm">
                <thead>
                  <tr>
                    <th className="border px-2 py-1 text-left">id</th>
                    <th className="border px-2 py-1 text-left">title</th>
                    <th className="border px-2 py-1 text-left">created_at</th>
                  </tr>
                </thead>
                <tbody>
                  {items.length === 0 && (
                    <tr>
                      <td className="border px-2 py-1" colSpan={3}>No rows yet.</td>
                    </tr>
                  )}
                  {items.map(item => (
                    <tr key={item.id}>
                      <td className="border px-2 py-1 font-mono text-xs">{item.id}</td>
                      <td className="border px-2 py-1">{item.title}</td>
                      <td className="border px-2 py-1 font-mono text-xs">{item.created_at}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">Insert Row (client-side POST)</h2>
          <p className="mt-2 text-xs text-gray-700">
            Runtime:
            {' '}
            <strong>React Client Component</strong>
            {' '}
            (runs in the browser, no page refresh).
          </p>
          <p className="mt-1 font-mono text-xs text-gray-700">
            Call:
            {' '}
            <code>supabase.from('demo_items').insert({'{'} title, user_id {'}'}).select('id')</code>
          </p>
          <form onSubmit={onSubmit} className="mt-3 flex gap-2">
            <input
              type="text"
              value={title}
              onChange={event => setTitle(event.target.value)}
              placeholder="Enter title"
              className="w-full rounded border px-3 py-2"
              required
            />
            <button
              type="submit"
              disabled={submitting}
              className="rounded bg-black px-4 py-2 text-white disabled:opacity-60"
            >
              {submitting ? 'Inserting...' : 'Insert'}
            </button>
          </form>
        </section>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">Calls Already Made</h2>
          <p className="mt-2 text-xs text-gray-700">
            Runtime:
            {' '}
            <strong>React Client Component</strong>
            {' '}
            call outcomes captured after each operation.
          </p>
          <div className="mt-3 overflow-auto">
            <table className="min-w-full border-collapse text-sm">
              <thead>
                <tr>
                  <th className="border px-2 py-1 text-left">Time</th>
                  <th className="border px-2 py-1 text-left">Call</th>
                  <th className="border px-2 py-1 text-left">Status</th>
                  <th className="border px-2 py-1 text-left">OK</th>
                  <th className="border px-2 py-1 text-left">Result</th>
                </tr>
              </thead>
              <tbody>
                {calls.length === 0 && (
                  <tr>
                    <td className="border px-2 py-1" colSpan={5}>No calls recorded yet.</td>
                  </tr>
                )}
                {calls.map((call, index) => (
                  <tr key={`${call.at}-${index}`}>
                    <td className="border px-2 py-1 font-mono text-xs">{call.at}</td>
                    <td className="border px-2 py-1 font-mono text-xs">{call.call}</td>
                    <td className="border px-2 py-1">{call.status ?? '-'}</td>
                    <td className="border px-2 py-1">{String(call.ok)}</td>
                    <td className="border px-2 py-1 font-mono text-xs">{call.result}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {error && <p className="text-sm text-red-700">{error}</p>}
      </div>
    </AuthGate>
  )
}
