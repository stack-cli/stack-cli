'use client'

import { ChangeEvent, FormEvent, useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import AuthGate from '@/components/auth/AuthGate'
import { getSupabaseBrowserClient } from '@/lib/supabase/client'

type UploadItem = {
  id: string
  file: File
  previewUrl: string
  status: 'pending' | 'uploading' | 'success' | 'error'
  result: string
  path: string
}

const MAX_SIZE_BYTES = 5 * 1024 * 1024
const ALLOWED_TYPES = new Set(['image/jpeg', 'image/png', 'image/webp', 'image/gif'])

function createUploadId(file: File) {
  return `${file.name}-${file.size}-${file.lastModified}-${Math.random().toString(36).slice(2, 8)}`
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

function sanitizeFileName(name: string) {
  return name.replace(/[^a-zA-Z0-9._-]/g, '_')
}

export default function StorageUploadClient() {
  const [items, setItems] = useState<UploadItem[]>([])
  const [error, setError] = useState<string | null>(null)
  const [uploading, setUploading] = useState(false)
  const router = useRouter()

  const configError = !process.env.NEXT_PUBLIC_SUPABASE_URL
    ? 'Missing NEXT_PUBLIC_SUPABASE_URL in .env.local'
    : !process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY
      ? 'Missing NEXT_PUBLIC_SUPABASE_ANON_KEY in .env.local'
      : null

  const hasPendingItems = useMemo(
    () => items.some((item) => item.status === 'pending' || item.status === 'error'),
    [items],
  )

  function onFilesSelected(event: ChangeEvent<HTMLInputElement>) {
    setError(null)
    const selected = Array.from(event.target.files ?? [])

    const nextItems: UploadItem[] = []
    const validationErrors: string[] = []

    for (const file of selected) {
      if (!ALLOWED_TYPES.has(file.type)) {
        validationErrors.push(`${file.name}: unsupported type ${file.type || '<unknown>'}`)
        continue
      }

      if (file.size > MAX_SIZE_BYTES) {
        validationErrors.push(`${file.name}: exceeds 5MB limit`)
        continue
      }

      nextItems.push({
        id: createUploadId(file),
        file,
        previewUrl: URL.createObjectURL(file),
        status: 'pending',
        result: 'Ready to upload',
        path: '',
      })
    }

    if (validationErrors.length > 0) {
      setError(validationErrors.join('; '))
    }

    setItems((prev) => [...prev, ...nextItems])
    event.target.value = ''
  }

  function removeItem(id: string) {
    setItems((prev) => {
      const match = prev.find((item) => item.id === id)
      if (match) URL.revokeObjectURL(match.previewUrl)
      return prev.filter((item) => item.id !== id)
    })
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()

    if (configError) {
      setError(configError)
      return
    }

    if (items.length === 0) {
      setError('Select at least one image')
      return
    }

    setUploading(true)
    setError(null)

    const supabase = getSupabaseBrowserClient()
    const { data } = await supabase.auth.getSession()
    const userId = data.session?.user?.id

    if (!userId) {
      setUploading(false)
      setError('No authenticated session found')
      return
    }

    for (const item of items) {
      if (item.status === 'success') continue

      setItems((prev) => prev.map((candidate) =>
        candidate.id === item.id
          ? { ...candidate, status: 'uploading', result: 'Uploading...' }
          : candidate,
      ))

      const safeName = sanitizeFileName(item.file.name)
      const path = `${userId}/${Date.now()}-${safeName}`

      const { error: uploadError } = await supabase
        .storage
        .from('demo_images')
        .upload(path, item.file, {
          contentType: item.file.type,
          upsert: false,
        })

      if (uploadError) {
        setItems((prev) => prev.map((candidate) =>
          candidate.id === item.id
            ? {
                ...candidate,
                status: 'error',
                result: `${uploadError.name}: ${uploadError.message}`,
              }
            : candidate,
        ))
      } else {
        setItems((prev) => prev.map((candidate) =>
          candidate.id === item.id
            ? {
                ...candidate,
                status: 'success',
                result: 'Uploaded',
                path,
              }
            : candidate,
        ))
      }
    }

    setUploading(false)
    router.refresh()
  }

  return (
    <AuthGate>
      <section className="card">
        <div className="row">
          <h2 style={{ margin: 0 }}>Upload Images</h2>
          <span className="badge badge-client">Client</span>
        </div>

        <p className="muted" style={{ marginTop: '0.5rem' }}>
          React client component with local file queue, then direct uploads via Supabase Storage JS.
        </p>
        <p className="mono" style={{ fontSize: '0.75rem' }}>
          Calls: supabase.storage.from('demo_images').upload(path, file, {'{'} upsert: false, contentType {'}'})
        </p>

        <form onSubmit={onSubmit} className="stack" style={{ marginTop: '0.75rem' }}>
          <div className="form-row">
            <label htmlFor="storage-files">Choose image files (jpg/png/webp/gif, max 5MB each)</label>
            <input
              id="storage-files"
              type="file"
              accept="image/jpeg,image/png,image/webp,image/gif"
              multiple
              onChange={onFilesSelected}
            />
          </div>

          <div className="table-wrap">
            <table className="table">
              <thead>
                <tr>
                  <th>Preview</th>
                  <th>Name</th>
                  <th>Type</th>
                  <th>Size</th>
                  <th>Status</th>
                  <th>Storage Path</th>
                  <th>Result</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {items.length === 0 && (
                  <tr>
                    <td colSpan={8}>No files selected yet.</td>
                  </tr>
                )}
                {items.map((item) => (
                  <tr key={item.id}>
                    <td>
                      <img
                        src={item.previewUrl}
                        alt={item.file.name}
                        style={{ width: 56, height: 56, objectFit: 'cover', borderRadius: 6, border: '1px solid #d1d5db' }}
                      />
                    </td>
                    <td>{item.file.name}</td>
                    <td className="mono" style={{ fontSize: '0.75rem' }}>{item.file.type}</td>
                    <td>{formatBytes(item.file.size)}</td>
                    <td>{item.status}</td>
                    <td className="mono" style={{ fontSize: '0.7rem' }}>{item.path || '-'}</td>
                    <td className="mono" style={{ fontSize: '0.7rem' }}>{item.result}</td>
                    <td>
                      <button
                        type="button"
                        onClick={() => removeItem(item.id)}
                        disabled={item.status === 'uploading' || uploading}
                      >
                        Remove
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="row">
            <button type="submit" className="btn" disabled={uploading || !hasPendingItems}>
              {uploading ? 'Uploading...' : 'Submit Uploads'}
            </button>
            <span className="muted" style={{ fontSize: '0.875rem' }}>
              Bucket: <code>demo_images</code> (private), folder prefix: <code>{'<' }user_id{'>'}/...</code>
            </span>
          </div>

          {error && <p className="error" style={{ margin: 0 }}>{error}</p>}
        </form>
      </section>
    </AuthGate>
  )
}
