'use client'

type AuthFlowInfoCardProps = {
  mode: 'login' | 'signup'
}

export default function AuthFlowInfoCard({ mode }: AuthFlowInfoCardProps) {
  const expectedCalls = mode === 'login'
    ? ['supabase.auth.signInWithPassword(...)', 'POST /auth/v1/token?grant_type=password']
    : [
        'supabase.auth.signUp(...)',
        'POST /auth/v1/signup',
        'supabase.auth.signInWithPassword(...) (demo auto-login)',
        'POST /auth/v1/token?grant_type=password',
      ]

  return (
    <section className="card">
      <h2 style={{ marginTop: 0 }}>Auth Flow</h2>
      <p className="muted">This card documents what this page will call.</p>
      <p style={{ marginBottom: '0.4rem' }}><strong>Expected calls on submit</strong></p>
      <ul style={{ marginTop: 0, paddingLeft: '1.2rem' }}>
        {expectedCalls.map((call) => (
          <li key={call} className="mono" style={{ fontSize: '0.85rem' }}>{call}</li>
        ))}
      </ul>
      <p className="muted" style={{ marginBottom: 0 }}>
        On success a JWT session is returned and the app redirects to <code>/</code>.
      </p>
    </section>
  )
}
