import SignupForm from '@/components/auth/SignupForm'

export default function SignupPage() {
  return (
    <div className="space-y-4">
      <h1 className="text-3xl font-semibold">Create account</h1>
      <p className="text-gray-700">
        Sign up using Supabase Auth on your local Stack instance.
      </p>
      <SignupForm />
    </div>
  )
}

export const metadata = {
  title: 'Sign up | My rari App',
  description: 'Create an account',
}
