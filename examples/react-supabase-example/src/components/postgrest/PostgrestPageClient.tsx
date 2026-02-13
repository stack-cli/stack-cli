'use client'

import { FormEvent, useEffect, useState } from 'react'
import AuthGate from '@/components/auth/AuthGate'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

type DemoItem = {
  id: string
  title: string
  created_at: string
}

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

  async function loadTopFive(accessToken: string) {
    const path = '/rest/v1/demo_items?select=id,title,created_at&order=created_at.desc&limit=5'
    const url = `${import.meta.env.VITE_SUPABASE_URL}${path}`

    try {
      const response = await fetch(url, {
        method: 'GET',
        headers: {
          apikey: import.meta.env.VITE_SUPABASE_ANON_KEY,
          Authorization: `Bearer ${accessToken}`,
          Accept: 'application/json',
        },
      })
      const bodyText = await response.text()
      recordCall({
        at: new Date().toISOString(),
        call: `GET ${path}`,
        status: response.status,
        ok: response.ok,
        result: bodyText.slice(0, 140) || '<empty>',
      })

      if (!response.ok) {
        setError(`List request failed with status ${response.status}`)
        return
      }

      const parsed = JSON.parse(bodyText) as DemoItem[]
      setItems(parsed)
    } catch (nextError) {
      recordCall({
        at: new Date().toISOString(),
        call: `GET ${path}`,
        status: null,
        ok: false,
        result: nextError instanceof Error ? nextError.message : 'Request failed',
      })
      setError(nextError instanceof Error ? nextError.message : 'List request failed')
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

      await loadTopFive(data.session.access_token)
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

    const path = '/rest/v1/demo_items'
    const url = `${import.meta.env.VITE_SUPABASE_URL}${path}`
    const payload = {
      title,
      user_id: data.session.user.id,
    }

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          apikey: import.meta.env.VITE_SUPABASE_ANON_KEY,
          Authorization: `Bearer ${data.session.access_token}`,
          'Content-Type': 'application/json',
          Prefer: 'return=representation',
        },
        body: JSON.stringify(payload),
      })
      const bodyText = await response.text()
      recordCall({
        at: new Date().toISOString(),
        call: `POST ${path}`,
        status: response.status,
        ok: response.ok,
        result: bodyText.slice(0, 140) || '<empty>',
      })

      if (!response.ok) {
        setSubmitting(false)
        setError(`Insert failed with status ${response.status}`)
        return
      }

      setTitle('')
      await loadTopFive(data.session.access_token)
    } catch (nextError) {
      recordCall({
        at: new Date().toISOString(),
        call: `POST ${path}`,
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
          <h2 className="text-xl font-medium">Calls This Page Will Make</h2>
          <ul className="mt-2 space-y-1 font-mono text-xs">
            <li>GET /rest/v1/demo_items?select=id,title,created_at&order=created_at.desc&limit=5</li>
            <li>POST /rest/v1/demo_items</li>
          </ul>
          <p className="mt-2 text-xs text-gray-700">
            Headers:
            {' '}
            <code>apikey</code>
            ,
            {' '}
            <code>Authorization: Bearer &lt;access_token&gt;</code>
            , and JSON content type for POST.
          </p>
        </section>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">Top 5 Rows (client-side GET)</h2>
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
