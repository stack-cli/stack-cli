'use client'

import { FormEvent, useState } from 'react'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

export default function SignupForm() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
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
    setMessage(null)

    const supabase = getSupabaseBrowserClient()
    const { data, error: signupError } = await supabase.auth.signUp({ email, password })

    if (signupError) {
      setLoading(false)
      setError(signupError.message)
      return
    }

    if (data.session) {
      setLoading(false)
      window.location.assign('/')
      return
    }

    const { data: loginData, error: loginError } = await supabase.auth.signInWithPassword({ email, password })
    setLoading(false)

    if (!loginError && loginData.session) {
      window.location.assign('/')
      return
    }

    setMessage('Account created. Sign in from /login if auto-login is disabled.')
  }

  return (
    <form onSubmit={onSubmit} className="form">
      <div className="form-row">
        <label htmlFor="signup-email">Email</label>
        <input
          id="signup-email"
          type="email"
          value={email}
          onChange={(event) => setEmail(event.target.value)}
          required
          className="input"
        />
      </div>

      <div className="form-row">
        <label htmlFor="signup-password">Password</label>
        <input
          id="signup-password"
          type="password"
          minLength={6}
          value={password}
          onChange={(event) => setPassword(event.target.value)}
          required
          className="input"
        />
      </div>

      <button type="submit" disabled={loading} className="btn">
        {loading ? 'Creating account...' : 'Create account'}
      </button>

      {error && <p className="error">{error}</p>}
      {message && <p className="success">{message}</p>}
    </form>
  )
}
