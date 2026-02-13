import Counter from '@/components/Counter'

// This is a React Server Component - runs on the server!
export default async function HomePage() {
// Fetch data on the server
const response = await fetch('https://api.github.com/repos/facebook/react')
const repoData = await response.json()

return (
  <div>
    <h1>Welcome to rari</h1>

    {/* Server-rendered content */}
    <div>
      <h2>React Repository Stats</h2>
      <p>Stars: {repoData.stargazers_count.toLocaleString()}</p>
      <p>Forks: {repoData.forks_count.toLocaleString()}</p>
      <p>Watchers: {repoData.watchers_count.toLocaleString()}</p>
      <p>Last updated: {new Date(repoData.updated_at).toLocaleDateString()}</p>
    </div>

    {/* Client Component */}
    <Counter />
  </div>
)
}

export const metadata = {
title: 'Home | React Supabase Example',
description: 'Welcome to my rari application',
}
