import { createClient, type SupabaseClient } from '@supabase/supabase-js'
import { setLastAuthCall } from '@/lib/supabase/flowState'

let browserClient: SupabaseClient | undefined

export function getSupabaseBrowserClient(): SupabaseClient {
  const supabaseUrl = import.meta.env.VITE_SUPABASE_URL
  const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY

  if (!supabaseUrl) {
    throw new Error('Missing VITE_SUPABASE_URL in environment')
  }

  if (!supabaseAnonKey) {
    throw new Error('Missing VITE_SUPABASE_ANON_KEY in environment')
  }

  if (!browserClient) {
    browserClient = createClient(supabaseUrl, supabaseAnonKey, {
      global: {
        fetch: async (input, init) => {
          const urlString = typeof input === 'string'
            ? input
            : input instanceof URL
              ? input.toString()
              : input.url
          const inputMethod = input instanceof Request ? input.method : undefined
          const method = init?.method ?? inputMethod ?? 'GET'

          try {
            const response = await fetch(input, init)
            const path = (() => {
              try {
                return new URL(urlString).pathname
              } catch {
                return urlString
              }
            })()

            if (path.includes('/auth/')) {
              setLastAuthCall({
                at: new Date().toISOString(),
                method,
                path,
                status: response.status,
                ok: response.ok,
              })
            }

            return response
          } catch (error) {
            const path = (() => {
              try {
                return new URL(urlString).pathname
              } catch {
                return urlString
              }
            })()

            if (path.includes('/auth/')) {
              setLastAuthCall({
                at: new Date().toISOString(),
                method,
                path,
                ok: false,
                error: error instanceof Error ? error.message : 'Request failed',
              })
            }

            throw error
          }
        },
      },
    })
  }

  return browserClient
}
