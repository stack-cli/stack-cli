'use client'

import { FormEvent, useState } from 'react'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

export default function LoginForm() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const configError = !process.env.NEXT_PUBLIC_SUPABASE_URL
    ? 'Missing NEXT_PUBLIC_SUPABASE_URL in .env.local'
    : !process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY
      ? 'Missing NEXT_PUBLIC_SUPABASE_ANON_KEY in .env.local'
      : null

  if (configError) {
    return <p className="error">{configError}</p>
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setLoading(true)
    setError(null)

    const supabase = getSupabaseBrowserClient()
    const { error: loginError } = await supabase.auth.signInWithPassword({ email, password })

    setLoading(false)

    if (loginError) {
      setError(loginError.message)
      return
    }

    window.location.assign('/')
  }

  return (
    <form onSubmit={onSubmit} className="form">
      <div className="form-row">
        <label htmlFor="login-email">Email</label>
        <input
          id="login-email"
          type="email"
          value={email}
          onChange={(event) => setEmail(event.target.value)}
          required
          className="input"
        />
      </div>

      <div className="form-row">
        <label htmlFor="login-password">Password</label>
        <input
          id="login-password"
          type="password"
          value={password}
          onChange={(event) => setPassword(event.target.value)}
          required
          className="input"
        />
      </div>

      <button type="submit" disabled={loading} className="btn">
        {loading ? 'Signing in...' : 'Sign in'}
      </button>

      {error && <p className="error">{error}</p>}
    </form>
  )
}
