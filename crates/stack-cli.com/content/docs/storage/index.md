# Storage

Stack ships a Supabase Storage API deployment so your app can serve and store files without wiring S3 by hand. By default the controller deploys MinIO in your namespace, generates credentials, and injects them plus a JWT secret into the storage pod.

## What the controller creates

- A `storage` Deployment using `supabase/storage-api` on port `5000`.
- A `storage-s3` Secret with generated AWS-compat credentials and endpoint/bucket settings.
- A `storage-auth` Secret with a random `AUTH_JWT_SECRET` (HS256).
- A MinIO Deployment/Service when you have not provided your own S3 secret.

## Customising with the CRD

Add a storage block to your `StackApp`:

```yaml
spec:
  storage:
    # Optional: expose storage via NodePort for local testing
    expose_storage_port: 30012
    # Optional: point at your own S3 secret and skip MinIO
    s3_secret_name: my-s3-secret
    install_minio: false
```

### Using the default MinIO

If you omit `s3_secret_name`, the controller:

- Creates `storage-s3` with defaults (bucket `supa-storage-bucket`, endpoint `http://minio:9000`, region `us-east-1`, path-style forced).
- Generates random access keys and S3 protocol keys.
- Deploys MinIO with those credentials.

### Using an external S3/MinIO

Create a secret (any name) in your app namespace with these keys:

- `STORAGE_S3_BUCKET`
- `STORAGE_S3_ENDPOINT` (e.g. `https://s3.amazonaws.com` or `http://minio.example:9000`)
- `STORAGE_S3_REGION`
- `STORAGE_S3_FORCE_PATH_STYLE` (`"true"` or `"false"`)
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `S3_PROTOCOL_ACCESS_KEY_ID`
- `S3_PROTOCOL_ACCESS_KEY_SECRET`

Then set `s3_secret_name` to that secret and `install_minio: false` if you don’t want the bundled MinIO.

## Database wiring

- Storage uses the `migrations-url` from the `database-urls` secret so it can run migrations and manage roles (`DB_INSTALL_ROLES=true`).
- JWT auth uses the generated `storage-auth` secret and HS256; override by patching that secret if you need to share a token across services.
- Supabase Storage keeps metadata in your app’s Postgres database under the `storage` schema (tables like `buckets`, `objects`, and policies). Plan migrations/backups accordingly—metadata and object store must stay in sync.
