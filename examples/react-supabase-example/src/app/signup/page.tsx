import SignupForm from '@/components/auth/SignupForm'
import AuthFlowInfoCard from '@/components/AuthFlowInfoCard'

export default function SignupPage() {
  return (
    <div className="space-y-4">
      <h1 className="text-3xl font-semibold">Create account</h1>
      <p className="text-gray-700">
        Sign up using Supabase Auth on your local Stack instance.
      </p>
      <SignupForm />
      <AuthFlowInfoCard mode="signup" />
    </div>
  )
}

export const metadata = {
  title: 'Sign up | React Supabase Example',
  description: 'Create an account',
}
