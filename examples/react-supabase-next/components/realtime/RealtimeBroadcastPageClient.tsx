'use client'

import { FormEvent, useEffect, useMemo, useRef, useState } from 'react'
import type { RealtimeChannel } from '@supabase/supabase-js'
import AuthGate from '@/components/auth/AuthGate'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

const CHANNEL_NAME = 'realtime-broadcast-demo'
const EVENT_NAME = 'message'
const MAX_LINES = 200

type ConnectionStatus = 'connecting' | 'subscribed' | 'closed' | 'errored'

export default function RealtimeBroadcastPageClient() {
  const [draft, setDraft] = useState('')
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [status, setStatus] = useState<ConnectionStatus>('connecting')
  const [lines, setLines] = useState<string[]>([
    'Realtime broadcast terminal ready.',
    `Channel: ${CHANNEL_NAME} | Event: ${EVENT_NAME}`,
    'Waiting for subscription...',
  ])
  const terminalRef = useRef<HTMLDivElement | null>(null)
  const channelRef = useRef<RealtimeChannel | null>(null)

  const statusLabel = useMemo(() => {
    if (status === 'subscribed') return 'Subscribed'
    if (status === 'connecting') return 'Connecting'
    if (status === 'closed') return 'Closed'
    return 'Errored'
  }, [status])

  function writeLine(message: string) {
    setLines((previous) => [...previous.slice(-MAX_LINES + 1), message])
  }

  useEffect(() => {
    if (!terminalRef.current) return
    terminalRef.current.scrollTop = terminalRef.current.scrollHeight
  }, [lines])

  useEffect(() => {
    const configError = !process.env.NEXT_PUBLIC_SUPABASE_URL
      ? 'Missing NEXT_PUBLIC_SUPABASE_URL in .env.local'
      : !process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY
        ? 'Missing NEXT_PUBLIC_SUPABASE_ANON_KEY in .env.local'
        : null

    if (configError) {
      setError(configError)
      setStatus('errored')
      writeLine(configError)
      return
    }

    const supabase = getSupabaseBrowserClient()
    let cancelled = false

    const channel = supabase
      .channel(CHANNEL_NAME)
      .on('broadcast', { event: EVENT_NAME }, ({ payload }) => {
        const body = typeof payload === 'object' ? JSON.stringify(payload) : String(payload ?? '')
        const time = new Date().toLocaleTimeString()
        writeLine(`[${time}] ${body}`)
      })
      .subscribe((nextStatus) => {
        if (cancelled) return

        if (nextStatus === 'SUBSCRIBED') {
          setStatus('subscribed')
          writeLine('Subscription active.')
          return
        }
        if (nextStatus === 'CLOSED') {
          setStatus('closed')
          writeLine('Subscription closed.')
          return
        }
        if (nextStatus === 'CHANNEL_ERROR' || nextStatus === 'TIMED_OUT') {
          const message = `Subscription failed: ${nextStatus}`
          setStatus('errored')
          setError(message)
          writeLine(message)
          return
        }

        setStatus('connecting')
      })

    channelRef.current = channel

    return () => {
      cancelled = true
      channelRef.current = null
      void supabase.removeChannel(channel)
    }
  }, [])

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const channel = channelRef.current
    const message = draft.trim()
    if (!channel || !message) return

    setSending(true)
    setError(null)

    const result = await channel.send({
      type: 'broadcast',
      event: EVENT_NAME,
      payload: {
        message,
        sent_at: new Date().toISOString(),
      },
    })

    if (result !== 'ok') {
      const nextError = `Broadcast send failed: ${result}`
      setError(nextError)
      writeLine(nextError)
      setSending(false)
      return
    }

    setDraft('')
    setSending(false)
  }

  return (
    <AuthGate>
      <div className="stack">
        <h1 style={{ margin: 0 }}>Realtime Broadcast</h1>
        <p className="muted">Send and receive broadcast messages via Supabase realtime subscriptions.</p>

        <section className="card">
          <div className="row">
            <h2 style={{ margin: 0 }}>Broadcast Stream</h2>
            <span className="badge badge-client">Client</span>
            <span className="badge">{statusLabel}</span>
          </div>
          <p className="mono" style={{ fontSize: '0.75rem', marginTop: '0.5rem' }}>
            Calls: supabase.channel('{CHANNEL_NAME}').on('broadcast', {'{'} event: '{EVENT_NAME}' {'}'}, ...).subscribe()
            and channel.send({'{'} type: 'broadcast', event: '{EVENT_NAME}', payload {'}'})
          </p>
          <div className="terminal-shell" ref={terminalRef}>
            {lines.map((line, index) => (
              <div key={`${index}-${line}`} className="terminal-line">{line}</div>
            ))}
          </div>
        </section>

        <section className="card">
          <div className="row">
            <h2 style={{ margin: 0 }}>Send Broadcast Event</h2>
            <span className="badge badge-client">Client</span>
          </div>
          <form onSubmit={onSubmit} className="row" style={{ marginTop: '0.75rem' }}>
            <input
              type="text"
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder="Enter event message"
              required
              className="input"
            />
            <button type="submit" disabled={sending || status !== 'subscribed'} className="btn">
              {sending ? 'Sending...' : 'Send'}
            </button>
          </form>
        </section>

        {error && <p className="error">{error}</p>}
      </div>
    </AuthGate>
  )
}
