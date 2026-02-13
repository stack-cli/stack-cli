import AuthGate from '@/components/auth/AuthGate'
import StackHealthClientCard from '@/components/StackHealthClientCard'
import { runStackHealthChecks } from '@/lib/supabase/api'

async function runServerChecks() {
  const baseUrl = process.env.SUPABASE_SERVER_URL
    ?? process.env.VITE_SUPABASE_URL
    ?? 'http://host.docker.internal:30010'
  const anonKey = process.env.VITE_SUPABASE_ANON_KEY ?? ''
  return runStackHealthChecks(baseUrl, anonKey)
}

// This is a React Server Component - runs on the server!
export default async function HomePage() {
  const serverRows = await runServerChecks()

  return (
    <AuthGate>
      <div className="space-y-6">
        <section>
          <h1 className="text-3xl font-semibold">React + Stack (Self-Hosted Supabase)</h1>
          <p className="mt-2 text-gray-700">
            This app demonstrates browser auth flows against Stack Auth at
            {' '}
            <code>http://localhost:30010/auth</code>
            .
          </p>
        </section>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">React Server Component Calling Stack APIs</h2>
          <p className="mt-2 text-sm text-gray-700">
            These calls run on the server during RSC render using
            {' '}
            <code>SUPABASE_SERVER_URL</code>
            {' '}
            (for dev containers use
            {' '}
            <code>http://host.docker.internal:30010</code>
            ).
          </p>
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
                {serverRows.map(row => (
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
        </section>

        <StackHealthClientCard />
      </div>
    </AuthGate>
  )
}

export const metadata = {
  title: 'Home | React Supabase Example',
  description: 'React app using Stack Auth as self-hosted Supabase backend',
}
