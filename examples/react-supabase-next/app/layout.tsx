import type { Metadata } from 'next'
import './globals.css'
import AppNav from '@/components/AppNav'

export const metadata: Metadata = {
  title: 'React Supabase Next Example',
  description: 'React app using Stack Auth as self-hosted Supabase backend',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <div className="shell">
          <AppNav />
          <main>{children}</main>
        </div>
      </body>
    </html>
  )
}
