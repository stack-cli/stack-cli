import LoginForm from '@/components/auth/LoginForm'

export default function LoginPage() {
  return (
    <div className="space-y-4">
      <h1 className="text-3xl font-semibold">Sign in</h1>
      <p className="text-gray-700">
        Log in using Supabase Auth on your local Stack instance.
      </p>
      <LoginForm />
    </div>
  )
}

export const metadata = {
  title: 'Login | React Supabase Example',
  description: 'Sign in to your account',
}
