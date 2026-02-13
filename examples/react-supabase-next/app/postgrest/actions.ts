'use server'

import { createClient } from '@supabase/supabase-js'

export type InsertDemoItemState = {
  ok: boolean
  message: string
  submittedAt: number
}

export const INSERT_DEMO_ITEM_INITIAL_STATE: InsertDemoItemState = {
  ok: false,
  message: '',
  submittedAt: 0,
}

export async function insertDemoItemServerAction(
  _prevState: InsertDemoItemState,
  formData: FormData,
): Promise<InsertDemoItemState> {
  const title = String(formData.get('title') ?? '').trim()
  const userId = String(formData.get('user_id') ?? '').trim()
  const accessToken = String(formData.get('access_token') ?? '').trim()

  if (!title) return { ok: false, message: 'Missing title', submittedAt: Date.now() }
  if (!userId || !accessToken) {
    return { ok: false, message: 'Missing authenticated session values', submittedAt: Date.now() }
  }

  const baseUrl = process.env.SUPABASE_SERVER_URL
    ?? process.env.NEXT_PUBLIC_SUPABASE_URL
    ?? 'http://host.docker.internal:30010'
  const anonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY

  if (!anonKey) {
    return { ok: false, message: 'Missing NEXT_PUBLIC_SUPABASE_ANON_KEY on server', submittedAt: Date.now() }
  }

  const supabase = createClient(baseUrl, anonKey, {
    global: {
      headers: { Authorization: `Bearer ${accessToken}` },
    },
  })

  const { data, error } = await supabase
    .from('demo_items')
    .insert({ title, user_id: userId })
    .select('id')
    .limit(1)

  if (error) {
    return {
      ok: false,
      message: `${error.code ?? 'error'}: ${error.message}`,
      submittedAt: Date.now(),
    }
  }

  return {
    ok: true,
    message: `${(data ?? []).length} row inserted (server action)`,
    submittedAt: Date.now(),
  }
}
