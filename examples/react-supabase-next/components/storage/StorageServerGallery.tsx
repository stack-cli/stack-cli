import { cookies } from 'next/headers'
import { createClient } from '@supabase/supabase-js'
import { sessionCookieNames } from '@/lib/supabase/sessionCookieConstants'

type GalleryItem = {
  path: string
  name: string
  createdAt: string
  size: number
  signedUrl: string
}

export default async function StorageServerGallery() {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get(sessionCookieNames.accessToken)?.value ?? ''
  const userId = cookieStore.get(sessionCookieNames.userId)?.value ?? ''

  if (!accessToken || !userId) {
    return (
      <section className="card">
        <div className="row">
          <h2 style={{ margin: 0 }}>Current User Thumbnails</h2>
          <span className="badge badge-server">Server</span>
        </div>
        <p className="muted" style={{ marginTop: '0.5rem' }}>
          React server component. Sign in and refresh this page once so the server can read the session cookie.
        </p>
      </section>
    )
  }

  const baseUrl = process.env.SUPABASE_SERVER_URL
    ?? process.env.NEXT_PUBLIC_SUPABASE_URL
    ?? 'http://host.docker.internal:30010'
  const anonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? ''

  if (!anonKey) {
    return (
      <section className="card">
        <p className="error">Missing NEXT_PUBLIC_SUPABASE_ANON_KEY on server.</p>
      </section>
    )
  }

  const supabase = createClient(baseUrl, anonKey, {
    global: {
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    },
  })

  const { data: objects, error: listError } = await supabase
    .storage
    .from('demo_images')
    .list(userId, {
      limit: 12,
      sortBy: { column: 'created_at', order: 'desc' },
    })

  if (listError) {
    return (
      <section className="card">
        <div className="row">
          <h2 style={{ margin: 0 }}>Current User Thumbnails</h2>
          <span className="badge badge-server">Server</span>
        </div>
        <p className="error">{listError.name}: {listError.message}</p>
      </section>
    )
  }

  const safeObjects = (objects ?? []).filter((item) => item.name)

  const results: GalleryItem[] = []
  for (const object of safeObjects) {
    const fullPath = `${userId}/${object.name}`
    const { data: signed, error: signError } = await supabase
      .storage
      .from('demo_images')
      .createSignedUrl(fullPath, 60 * 10)

    if (signError || !signed?.signedUrl) {
      continue
    }

    results.push({
      path: fullPath,
      name: object.name,
      createdAt: object.created_at ?? '-',
      size: object.metadata?.size ?? 0,
      signedUrl: signed.signedUrl,
    })
  }

  return (
    <section className="card">
      <div className="row">
        <h2 style={{ margin: 0 }}>Current User Thumbnails</h2>
        <span className="badge badge-server">Server</span>
      </div>
      <p className="muted" style={{ marginTop: '0.5rem' }}>
        React server component listing private objects and creating signed URLs on the server.
      </p>
      <p className="mono" style={{ fontSize: '0.75rem' }}>
        Calls: supabase.storage.from('demo_images').list(userId) and createSignedUrl(path, 600)
      </p>

      {results.length === 0 && <p className="muted">No images uploaded for this user yet.</p>}

      {results.length > 0 && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
            gap: '0.75rem',
            marginTop: '0.75rem',
          }}
        >
          {results.map((item) => (
            <figure key={item.path} style={{ margin: 0, display: 'grid', gap: '0.4rem' }}>
              <img
                src={item.signedUrl}
                alt={item.name}
                style={{ width: '100%', aspectRatio: '1 / 1', objectFit: 'cover', borderRadius: 8, border: '1px solid #d1d5db' }}
              />
              <figcaption className="mono" style={{ fontSize: '0.68rem', wordBreak: 'break-all' }}>
                <div>{item.name}</div>
                <div>{item.createdAt}</div>
                <div>{item.size} bytes</div>
              </figcaption>
            </figure>
          ))}
        </div>
      )}
    </section>
  )
}
