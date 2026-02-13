'use client'

import { useEffect, useState } from 'react'
import { runStackHealthChecks, type HealthRow } from '@/lib/supabase/api'

export default function StackHealthClientCard() {
  const [rows, setRows] = useState<HealthRow[]>([])
  const [loading, setLoading] = useState(true)

  const baseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL
  const anonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY
  const configError = !baseUrl
    ? 'Missing NEXT_PUBLIC_SUPABASE_URL in .env.local'
    : !anonKey
      ? 'Missing NEXT_PUBLIC_SUPABASE_ANON_KEY in .env.local'
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
    <section className="card">
      <div className="row">
        <h2 style={{ margin: 0 }}>Server APIs from Client Component</h2>
        <span className="badge badge-client">Client</span>
      </div>
      <p className="muted">
        This is a React client component. It runs in the browser after hydration and calls Stack endpoints directly.
      </p>
      {configError && <p className="error">{configError}</p>}
      {loading && !configError && <p className="muted">Running checks...</p>}
      {!loading && !configError && (
        <div className="table-wrap">
          <table className="table">
            <thead>
              <tr>
                <th>Component</th>
                <th>Call</th>
                <th>Status</th>
                <th>OK</th>
                <th>Result</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={`${row.component}-${row.call}`}>
                  <td>{row.component}</td>
                  <td className="mono">{row.call}</td>
                  <td>{row.status ?? '-'}</td>
                  <td>{String(row.ok)}</td>
                  <td className="mono" style={{ fontSize: '0.75rem' }}>{row.result}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}
