'use client'

import { FormEvent, useState } from 'react'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

export default function SignupForm() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const configError = !import.meta.env.VITE_SUPABASE_URL
    ? 'Missing VITE_SUPABASE_URL in .env.local'
    : !import.meta.env.VITE_SUPABASE_ANON_KEY
      ? 'Missing VITE_SUPABASE_ANON_KEY in .env.local'
      : null

  if (configError) {
    return <p className="text-sm text-red-700">{configError}</p>
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setLoading(true)
    setError(null)
    setMessage(null)

    const supabase = getSupabaseBrowserClient()
    const { data, error: signupError } = await supabase.auth.signUp({
      email,
      password,
    })

    setLoading(false)

    if (signupError) {
      setError(signupError.message)
      return
    }

    if (data.session) {
      window.location.assign('/')
      return
    }

    const { data: loginData, error: loginError } = await supabase.auth.signInWithPassword({
      email,
      password,
    })

    if (!loginError && loginData.session) {
      window.location.assign('/')
      return
    }

    setMessage('Account created. Sign in from /login if auto-login is disabled.')
  }

  return (
    <form onSubmit={onSubmit} className="space-y-4 max-w-md">
      <div>
        <label htmlFor="signup-email" className="block text-sm font-medium">
          Email
        </label>
        <input
          id="signup-email"
          type="email"
          value={email}
          onChange={event => setEmail(event.target.value)}
          required
          className="mt-1 w-full rounded border px-3 py-2"
        />
      </div>

      <div>
        <label htmlFor="signup-password" className="block text-sm font-medium">
          Password
        </label>
        <input
          id="signup-password"
          type="password"
          value={password}
          onChange={event => setPassword(event.target.value)}
          required
          minLength={6}
          className="mt-1 w-full rounded border px-3 py-2"
        />
      </div>

      <button
        type="submit"
        disabled={loading}
        className="rounded bg-black px-4 py-2 text-white disabled:opacity-60"
      >
        {loading ? 'Creating account...' : 'Create account'}
      </button>

      {error && <p className="text-sm text-red-700">{error}</p>}
      {message && <p className="text-sm text-green-700">{message}</p>}
    </form>
  )
}
