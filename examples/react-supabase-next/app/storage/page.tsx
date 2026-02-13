import StorageServerGallery from '@/components/storage/StorageServerGallery'
import StorageUploadClient from '@/components/storage/StorageUploadClient'

export default function StoragePage() {
  return (
    <div className="stack">
      <section>
        <h1 style={{ margin: 0 }}>Storage</h1>
        <p className="muted">
          Private image upload demo using Supabase Storage: client-side upload queue and server-side thumbnail rendering.
        </p>
      </section>

      <StorageServerGallery />
      <StorageUploadClient />
    </div>
  )
}
