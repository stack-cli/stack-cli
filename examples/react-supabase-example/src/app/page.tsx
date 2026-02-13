import AuthGate from '@/components/auth/AuthGate'
import Counter from '@/components/Counter'

// This is a React Server Component - runs on the server!
export default async function HomePage() {
// Fetch data on the server
const response = await fetch('https://api.github.com/repos/facebook/react')
const repoData = await response.json()

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
          <h2 className="text-xl font-medium">Server-rendered Reference Data</h2>
          <p className="mt-2">React repo stars: {repoData.stargazers_count.toLocaleString()}</p>
          <p>Forks: {repoData.forks_count.toLocaleString()}</p>
          <p>Watchers: {repoData.watchers_count.toLocaleString()}</p>
          <p>Last updated: {new Date(repoData.updated_at).toLocaleDateString()}</p>
        </section>

        <section className="rounded-lg border p-4">
          <h2 className="text-xl font-medium">Client-side Interactivity</h2>
          <Counter />
        </section>
      </div>
    </AuthGate>
  )
}

export const metadata = {
  title: 'Home | React Supabase Example',
  description: 'React app using Stack Auth as self-hosted Supabase backend',
}
