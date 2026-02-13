import type { LayoutProps } from 'rari'
import AppNav from '@/components/AppNav'

export default function RootLayout({ children }: LayoutProps) {
  return (
    <div className="mx-auto max-w-4xl p-6 space-y-8">
      <AppNav />
      <main>{children}</main>
    </div>
  )
}

export const metadata = {
  title: 'React Supabase Example',
  description: 'Built with rari',
}
