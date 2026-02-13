'use client'

import { useEffect, useState } from 'react'

type HealthRow = {
  component: string
  call: string
  status: number | null
  ok: boolean
  result: string
}

function getChecks(baseUrl: string, anonKey: string) {
  return [
    {
      component: 'auth',
      method: 'GET',
      path: '/auth/v1/health',
      headers: {} as Record<string, string>,
    },
    {
      component: 'auth-settings',
      method: 'GET',
      path: '/auth/v1/settings',
      headers: {} as Record<string, string>,
    },
    {
      component: 'rest',
      method: 'GET',
      path: '/rest/v1/',
      headers: {
        apikey: anonKey,
        Authorization: `Bearer ${anonKey}`,
        Accept: 'application/openapi+json',
      } as Record<string, string>,
    },
    {
      component: 'storage',
      method: 'GET',
      path: '/storage/v1/bucket',
      headers: {
        apikey: anonKey,
        Authorization: `Bearer ${anonKey}`,
      } as Record<string, string>,
    },
  ].map(check => ({
    ...check,
    url: `${baseUrl}${check.path}`,
  }))
}

export default function StackHealthClientCard() {
  const [rows, setRows] = useState<HealthRow[]>([])
  const [loading, setLoading] = useState(true)

  const baseUrl = import.meta.env.VITE_SUPABASE_URL
  const anonKey = import.meta.env.VITE_SUPABASE_ANON_KEY
  const configError = !baseUrl
    ? 'Missing VITE_SUPABASE_URL in .env.local'
    : !anonKey
      ? 'Missing VITE_SUPABASE_ANON_KEY in .env.local'
      : null

  useEffect(() => {
    if (configError || !baseUrl || !anonKey) {
      setLoading(false)
      return
    }

    const run = async () => {
      const checks = getChecks(baseUrl, anonKey)
      const results = await Promise.all(checks.map(async (check) => {
        try {
          const response = await fetch(check.url, {
            method: check.method,
            headers: check.headers,
          })
          const text = await response.text()
          return {
            component: check.component,
            call: `${check.method} ${check.path}`,
            status: response.status,
            ok: response.ok,
            result: text.slice(0, 140) || '<empty>',
          } satisfies HealthRow
        } catch (error) {
          return {
            component: check.component,
            call: `${check.method} ${check.path}`,
            status: null,
            ok: false,
            result: error instanceof Error ? error.message : 'Request failed',
          } satisfies HealthRow
        }
      }))

      setRows(results)
      setLoading(false)
    }

    run().catch(() => {
      setLoading(false)
    })
  }, [anonKey, baseUrl, configError])

  return (
    <section className="rounded-lg border p-4">
      <h2 className="text-xl font-medium">React Client Component Calling Stack APIs</h2>
      <p className="mt-2 text-sm text-gray-700">
        These calls run in the browser after hydration using
        {' '}
        <code>VITE_SUPABASE_URL</code>
        .
      </p>
      {configError && <p className="mt-3 text-sm text-red-700">{configError}</p>}
      {loading && !configError && <p className="mt-3 text-sm text-gray-700">Running checks...</p>}
      {!loading && !configError && (
        <div className="mt-3 overflow-auto">
          <table className="min-w-full border-collapse text-sm">
            <thead>
              <tr>
                <th className="border px-2 py-1 text-left">Component</th>
                <th className="border px-2 py-1 text-left">Call</th>
                <th className="border px-2 py-1 text-left">Status</th>
                <th className="border px-2 py-1 text-left">OK</th>
                <th className="border px-2 py-1 text-left">Result</th>
              </tr>
            </thead>
            <tbody>
              {rows.map(row => (
                <tr key={`${row.component}-${row.call}`}>
                  <td className="border px-2 py-1">{row.component}</td>
                  <td className="border px-2 py-1 font-mono">{row.call}</td>
                  <td className="border px-2 py-1">{row.status ?? '-'}</td>
                  <td className="border px-2 py-1">{String(row.ok)}</td>
                  <td className="border px-2 py-1 font-mono text-xs">{row.result}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}
