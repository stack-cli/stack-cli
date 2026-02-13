'use client'

import { useActionState, useEffect, useState } from 'react'
import AuthGate from '@/components/auth/AuthGate'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'
import { listDemoItems, type DemoItem } from '@/lib/supabase/api'
import {
  INSERT_DEMO_ITEM_INITIAL_STATE,
  insertDemoItemServerAction,
} from '@/app/postgrest/actions'

export default function PostgrestPageClient() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [userId, setUserId] = useState('')
  const [accessToken, setAccessToken] = useState('')
  const [items, setItems] = useState<DemoItem[]>([])
  const [insertState, insertAction, insertPending] = useActionState(
    insertDemoItemServerAction,
    INSERT_DEMO_ITEM_INITIAL_STATE,
  )

  async function loadTopFive() {
    try {
      const supabase = getSupabaseBrowserClient()
      const { items: nextItems, ok, result } = await listDemoItems(supabase, 5)
      setItems(nextItems)

      if (!ok) {
        setError(result)
      }
    } catch (error) {
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

      setUserId(data.session.user.id)
      setAccessToken(data.session.access_token)
      await loadTopFive()
      setLoading(false)
    }

    run().catch((nextError: unknown) => {
      setError(nextError instanceof Error ? nextError.message : 'PostgREST call failed')
      setLoading(false)
    })
  }, [])

  useEffect(() => {
    if (!insertState.submittedAt) {
      return
    }

    if (!insertState.ok) {
      setError(insertState.message)
      return
    }

    setError(null)
    void loadTopFive()
  }, [insertState])

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
          <div className="flex items-center gap-2">
            <h2 className="text-xl font-medium">Top 5 Rows</h2>
            <span className="rounded border border-blue-300 bg-blue-50 px-2 py-0.5 text-xs font-medium text-blue-700">
              Client
            </span>
          </div>
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
          <div className="flex items-center gap-2">
            <h2 className="text-xl font-medium">Insert Row</h2>
            <span className="rounded border border-emerald-300 bg-emerald-50 px-2 py-0.5 text-xs font-medium text-emerald-700">
              Server
            </span>
          </div>
          <p className="mt-2 text-xs text-gray-700">
            Runtime:
            {' '}
            <strong>React Server Action</strong>
            {' '}
            for insert, triggered by a client form submission (no page refresh).
          </p>
          <p className="mt-1 font-mono text-xs text-gray-700">
            Call:
            {' '}
            <code>insertDemoItemServerAction -&gt; supabase.from('demo_items').insert({'{'} title, user_id {'}'}).select('id')</code>
          </p>
          <form action={insertAction} className="mt-3 flex gap-2">
            <input
              type="text"
              name="title"
              placeholder="Enter title"
              className="w-full rounded border px-3 py-2"
              required
            />
            <input type="hidden" name="user_id" value={userId} />
            <input type="hidden" name="access_token" value={accessToken} />
            <button
              type="submit"
              disabled={insertPending || !userId || !accessToken}
              className="rounded bg-black px-4 py-2 text-white disabled:opacity-60"
            >
              {insertPending ? 'Inserting...' : 'Insert'}
            </button>
          </form>
        </section>

        {error && <p className="text-sm text-red-700">{error}</p>}
      </div>
    </AuthGate>
  )
}
