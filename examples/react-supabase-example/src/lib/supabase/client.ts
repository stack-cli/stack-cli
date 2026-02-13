import { createClient, type SupabaseClient } from '@supabase/supabase-js'

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
    browserClient = createClient(supabaseUrl, supabaseAnonKey)
  }

  return browserClient
}
