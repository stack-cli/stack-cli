import type { LayoutProps } from 'rari'
import AuthStatus from '@/components/auth/AuthStatus'

export default function RootLayout({ children }: LayoutProps) {
  return (
    <div className="mx-auto max-w-4xl p-6 space-y-8">
      <nav className="flex flex-wrap items-center gap-4 border-b pb-4">
        <a href="/">Home</a>
        <a href="/about">About</a>
        <a href="/blog">Blog</a>
        <a href="/signup">Sign up</a>
        <a href="/login">Login</a>
        <div className="ml-auto">
          <AuthStatus />
        </div>
      </nav>
      <main>{children}</main>
    </div>
  )
}

export const metadata = {
  title: 'React Supabase Example',
  description: 'Built with rari',
}
