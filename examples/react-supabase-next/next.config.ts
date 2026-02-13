import type { NextConfig } from 'next'

const defaultAllowedOrigins = ['localhost:30010', 'localhost']
const extraAllowedOrigins = (process.env.NEXT_SERVER_ACTION_ALLOWED_ORIGINS ?? '')
  .split(',')
  .map(value => value.trim())
  .filter(Boolean)

const nextConfig: NextConfig = {
  output: 'standalone',
  experimental: {
    serverActions: {
      allowedOrigins: [...new Set([...defaultAllowedOrigins, ...extraAllowedOrigins])],
    },
  },
}

export default nextConfig
