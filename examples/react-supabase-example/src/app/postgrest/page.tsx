'use client'

import { useEffect, useState } from 'react'
import AuthGate from '@/components/auth/AuthGate'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

type PostgrestCall = {
  status: number
  bodyPreview: string
}

export default function PostgrestPage() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<PostgrestCall | null>(null)

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

      const restUrl = `${import.meta.env.VITE_SUPABASE_URL}/rest/v1/`
      const response = await fetch(restUrl, {
        method: 'GET',
        headers: {
          apikey: import.meta.env.VITE_SUPABASE_ANON_KEY,
          Authorization: `Bearer ${data.session.access_token}`,
          Accept: 'application/openapi+json',
        },
      })

      const bodyText = await response.text()
      setResult({
        status: response.status,
        bodyPreview: bodyText.slice(0, 600),
      })
      setLoading(false)
    }

    run().catch((nextError: unknown) => {
      setError(nextError instanceof Error ? nextError.message : 'PostgREST call failed')
      setLoading(false)
    })
  }, [])

  return (
    <AuthGate>
      <div className="space-y-4">
        <h1 className="text-3xl font-semibold">PostgREST</h1>
        <p className="text-gray-700">
          This page performs an authenticated request to
          {' '}
          <code>/rest/v1/</code>
          {' '}
          using your current auth session token.
        </p>

        {loading && <p className="text-sm text-gray-700">Calling PostgREST...</p>}
        {error && <p className="text-sm text-red-700">{error}</p>}
        {result && (
          <div className="rounded-lg border bg-white p-4">
            <p className="font-mono text-sm">status={result.status}</p>
            <pre className="mt-3 overflow-auto rounded bg-slate-50 p-3 text-xs">
              {result.bodyPreview}
            </pre>
          </div>
        )}
      </div>
    </AuthGate>
  )
}

export const metadata = {
  title: 'Postgrest | React Supabase Example',
  description: 'Authenticated PostgREST call example',
}

