import AuthGate from '@/components/auth/AuthGate'
import StackHealthClientCard from '@/components/StackHealthClientCard'

type HealthRow = {
  component: string
  call: string
  status: number | null
  ok: boolean
  result: string
}

async function runServerChecks(): Promise<HealthRow[]> {
  const baseUrl = process.env.VITE_SUPABASE_URL ?? 'http://localhost:30010'
  const anonKey = process.env.VITE_SUPABASE_ANON_KEY ?? ''

  const checks = [
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
      headers: anonKey
        ? ({
            apikey: anonKey,
            Authorization: `Bearer ${anonKey}`,
            Accept: 'application/openapi+json',
          } as Record<string, string>)
        : ({} as Record<string, string>),
    },
    {
      component: 'storage',
      method: 'GET',
      path: '/storage/v1/bucket',
      headers: anonKey
        ? ({
            apikey: anonKey,
            Authorization: `Bearer ${anonKey}`,
          } as Record<string, string>)
        : ({} as Record<string, string>),
    },
  ]

  return Promise.all(checks.map(async check => {
    const url = `${baseUrl}${check.path}`
    try {
      const response = await fetch(url, {
        method: check.method,
        headers: check.headers,
        cache: 'no-store',
      })
      const text = await response.text()
      return {
        component: check.component,
        call: `${check.method} ${check.path}`,
        status: response.status,
        ok: response.ok,
        result: text.slice(0, 140) || '<empty>',
      }
    } catch (error) {
      return {
        component: check.component,
        call: `${check.method} ${check.path}`,
        status: null,
        ok: false,
        result: error instanceof Error ? error.message : 'Request failed',
      }
    }
  }))
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
          <h2 className="text-xl font-medium">Server-Side Stack Health</h2>
          <p className="mt-2 text-sm text-gray-700">
            Calls are executed during server render.
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
