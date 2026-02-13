'use client'

import { useEffect, useState } from 'react'
import { runStackHealthChecks, type HealthRow } from '@/lib/supabase/api'

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
      const results = await runStackHealthChecks(baseUrl, anonKey)
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
