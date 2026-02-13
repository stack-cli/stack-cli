import AuthFlowInfoCard from '@/components/AuthFlowInfoCard'
import SignupForm from '@/components/auth/SignupForm'

export default function SignupPage() {
  return (
    <div className="stack">
      <section>
        <h1 style={{ margin: 0 }}>Create account</h1>
        <p className="muted">Sign up using Supabase Auth on your local Stack instance.</p>
      </section>
      <SignupForm />
      <AuthFlowInfoCard mode="signup" />
    </div>
  )
}
