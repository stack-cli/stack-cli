import type { SupabaseClient } from '@supabase/supabase-js'

export type HealthRow = {
  component: string
  call: string
  status: number | null
  ok: boolean
  result: string
}

export type DemoItem = {
  id: string
  title: string
  created_at: string
}

type HealthCheck = {
  component: string
  method: 'GET'
  path: string
  headers: Record<string, string>
}

function createHealthChecks(anonKey: string): HealthCheck[] {
  const authHeaders: Record<string, string> = anonKey
    ? {
        apikey: anonKey,
        Authorization: `Bearer ${anonKey}`,
      }
    : {}

  return [
    {
      component: 'auth',
      method: 'GET',
      path: '/auth/v1/health',
      headers: {},
    },
    {
      component: 'auth-settings',
      method: 'GET',
      path: '/auth/v1/settings',
      headers: {},
    },
    {
      component: 'rest',
      method: 'GET',
      path: '/rest/v1/',
      headers: (anonKey
        ? {
            ...authHeaders,
            Accept: 'application/openapi+json',
          }
        : {
            Accept: 'application/openapi+json',
          }) as Record<string, string>,
    },
    {
      component: 'realtime',
      method: 'GET',
      path: '/realtime/v1/api/health',
      headers: authHeaders as Record<string, string>,
    },
    {
      component: 'storage',
      method: 'GET',
      path: '/storage/v1/bucket',
      headers: authHeaders as Record<string, string>,
    },
  ]
}

export async function runStackHealthChecks(
  baseUrl: string,
  anonKey: string,
): Promise<HealthRow[]> {
  const checks = createHealthChecks(anonKey)

  return Promise.all(checks.map(async check => {
    const url = `${baseUrl}${check.path}`
    try {
      const response = await fetch(url, {
        method: check.method,
        headers: check.headers,
        cache: 'no-store',
      })
      const text = await response.text()
      return {
        component: check.component,
        call: `${check.method} ${check.path}`,
        status: response.status,
        ok: response.ok,
        result: text.slice(0, 140) || '<empty>',
      }
    } catch (error) {
      return {
        component: check.component,
        call: `${check.method} ${check.path}`,
        status: null,
        ok: false,
        result: error instanceof Error ? error.message : 'Request failed',
      }
    }
  }))
}

export async function listDemoItems(
  supabase: SupabaseClient,
  limit = 5,
): Promise<{ items: DemoItem[]; ok: boolean; result: string }> {
  const { data, error } = await supabase
    .from('demo_items')
    .select('id,title,created_at')
    .order('created_at', { ascending: false })
    .limit(limit)

  if (error) {
    return {
      items: [],
      ok: false,
      result: `${error.code ?? 'error'}: ${error.message}`,
    }
  }

  return {
    items: (data ?? []) as DemoItem[],
    ok: true,
    result: `${(data ?? []).length} row(s) returned`,
  }
}

export async function insertDemoItem(
  supabase: SupabaseClient,
  title: string,
  userId: string,
): Promise<{ ok: boolean; result: string }> {
  const { data, error } = await supabase
    .from('demo_items')
    .insert({ title, user_id: userId })
    .select('id')
    .limit(1)

  if (error) {
    return {
      ok: false,
      result: `${error.code ?? 'error'}: ${error.message}`,
    }
  }

  return {
    ok: true,
    result: `${(data ?? []).length} row inserted`,
  }
}
