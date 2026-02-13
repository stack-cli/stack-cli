import AuthFlowInfoCard from '@/components/AuthFlowInfoCard'
import LoginForm from '@/components/auth/LoginForm'

export default function LoginPage() {
  return (
    <div className="stack">
      <section>
        <h1 style={{ margin: 0 }}>Sign in</h1>
        <p className="muted">Log in using Supabase Auth on your local Stack instance.</p>
      </section>
      <LoginForm />
      <AuthFlowInfoCard mode="login" />
    </div>
  )
}
