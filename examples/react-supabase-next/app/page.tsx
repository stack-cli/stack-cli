import AuthGate from '@/components/auth/AuthGate'
import StackHealthClientCard from '@/components/StackHealthClientCard'
import { runStackHealthChecks } from '@/lib/supabase/api'

async function runServerChecks() {
  const baseUrl = process.env.SUPABASE_SERVER_URL
    ?? process.env.NEXT_PUBLIC_SUPABASE_URL
    ?? 'http://host.docker.internal:30010'
  const anonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? ''
  return runStackHealthChecks(baseUrl, anonKey)
}

export default async function HomePage() {
  const serverRows = await runServerChecks()

  return (
    <AuthGate>
      <div className="stack">
        <section>
          <h1 style={{ margin: 0 }}>React + Stack (Self-Hosted Supabase)</h1>
          <p className="muted">
            Example app showing auth, PostgREST, and realtime features with Stack exposed at <code>http://localhost:30010</code>.
          </p>
        </section>

        <section className="card">
          <div className="row">
            <h2 style={{ margin: 0 }}>Server APIs from Server Component</h2>
            <span className="badge badge-server">Server</span>
          </div>
          <p className="muted">
            This is a React server component. It runs on the server during render and calls Stack APIs with <code>SUPABASE_SERVER_URL</code>.
          </p>
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
                {serverRows.map((row) => (
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
        </section>

        <StackHealthClientCard />
      </div>
    </AuthGate>
  )
}
