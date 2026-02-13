'use client'

import type { Session } from '@supabase/supabase-js'
import { sessionCookieNames } from '@/lib/supabase/sessionCookieConstants'

const ONE_DAY_SECONDS = 60 * 60 * 24

function setCookie(name: string, value: string, maxAgeSeconds: number) {
  document.cookie = `${name}=${encodeURIComponent(value)}; Path=/; Max-Age=${maxAgeSeconds}; SameSite=Lax`
}

function clearCookie(name: string) {
  document.cookie = `${name}=; Path=/; Max-Age=0; SameSite=Lax`
}

export function syncSessionCookies(session: Session | null) {
  if (!session?.access_token || !session.user?.id) {
    clearCookie(ACCESS_TOKEN_COOKIE)
    clearCookie(USER_ID_COOKIE)
    return
  }

  setCookie(ACCESS_TOKEN_COOKIE, session.access_token, ONE_DAY_SECONDS)
  setCookie(USER_ID_COOKIE, session.user.id, ONE_DAY_SECONDS)
}

export function clearSessionCookies() {
  clearCookie(ACCESS_TOKEN_COOKIE)
  clearCookie(USER_ID_COOKIE)
}

const ACCESS_TOKEN_COOKIE = sessionCookieNames.accessToken
const USER_ID_COOKIE = sessionCookieNames.userId
