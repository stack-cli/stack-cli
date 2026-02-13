import type { LayoutProps } from 'rari'

export default function RootLayout({ children }: LayoutProps) {
return (
  <div>
    <nav>
      <a href="/">Home</a>
      <a href="/about">About</a>
    </nav>
    <main>{children}</main>
  </div>
)
}

export const metadata = {
title: 'My rari App',
description: 'Built with rari',
}