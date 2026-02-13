'use client'

import { useActionState, useEffect, useState } from 'react'
import AuthGate from '@/components/auth/AuthGate'
import { listDemoItems, type DemoItem } from '@/lib/supabase/api'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'
import {
  type InsertDemoItemState,
  insertDemoItemServerAction,
} from '@/app/postgrest/actions'

const INSERT_DEMO_ITEM_INITIAL_STATE: InsertDemoItemState = {
  ok: false,
  message: '',
  submittedAt: 0,
}

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
      if (!ok) setError(result)
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : 'List request failed')
    }
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
    if (!insertState.submittedAt) return
    if (!insertState.ok) {
      setError(insertState.message)
      return
    }
    setError(null)
    void loadTopFive()
  }, [insertState])

  return (
    <AuthGate>
      <div className="stack">
        <h1 style={{ margin: 0 }}>PostgREST</h1>
        <p className="muted">List is client-side; insert runs through a React server action.</p>

        <section className="card">
          <div className="row">
            <h2 style={{ margin: 0 }}>Top 5 Rows</h2>
            <span className="badge badge-client">Client</span>
          </div>
          <p className="muted" style={{ marginTop: '0.5rem' }}>
            React client component calling PostgREST via Supabase JS.
          </p>
          <p className="mono" style={{ fontSize: '0.75rem' }}>
            Calls: supabase.from('demo_items').select('id,title,created_at').order('created_at', desc).limit(5)
          </p>
          {loading && <p className="muted">Loading rows...</p>}
          {!loading && (
            <div className="table-wrap">
              <table className="table">
                <thead>
                  <tr>
                    <th>id</th>
                    <th>title</th>
                    <th>created_at</th>
                  </tr>
                </thead>
                <tbody>
                  {items.length === 0 && (
                    <tr>
                      <td colSpan={3}>No rows yet.</td>
                    </tr>
                  )}
                  {items.map((item) => (
                    <tr key={item.id}>
                      <td className="mono" style={{ fontSize: '0.75rem' }}>{item.id}</td>
                      <td>{item.title}</td>
                      <td className="mono" style={{ fontSize: '0.75rem' }}>{item.created_at}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>

        <section className="card">
          <div className="row">
            <h2 style={{ margin: 0 }}>Insert Row</h2>
            <span className="badge badge-server">Server</span>
          </div>
          <p className="muted" style={{ marginTop: '0.5rem' }}>
            React server action receives form data and writes through Supabase JS on the server.
          </p>
          <p className="mono" style={{ fontSize: '0.75rem' }}>
            Calls: insertDemoItemServerAction -&gt; supabase.from('demo_items').insert({'{'} title, user_id {'}'}).select('id')
          </p>
          <form action={insertAction} className="row" style={{ marginTop: '0.75rem' }}>
            <input
              type="text"
              name="title"
              placeholder="Enter title"
              className="input"
              required
            />
            <input type="hidden" name="user_id" value={userId} />
            <input type="hidden" name="access_token" value={accessToken} />
            <button type="submit" className="btn" disabled={insertPending || !userId || !accessToken}>
              {insertPending ? 'Inserting...' : 'Insert'}
            </button>
          </form>
        </section>

        {error && <p className="error">{error}</p>}
      </div>
    </AuthGate>
  )
}
