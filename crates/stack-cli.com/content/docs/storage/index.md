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

## Quick local test (demo manifest)

With the demo manifest (`expose_storage_port: 30012` and `danger_override_jwt_secret` set), you can hit the Storage API directly:

```bash
curl --location --request POST 'http://host.docker.internal:30012/bucket' \
  --header 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoic2VydmljZV9yb2xlIiwiaWF0IjoxNjEzNTMxOTg1LCJleHAiOjE5MjkxMDc5ODV9.th84OKK0Iz8QchDyXZRrojmKSEZ-OuitQm_5DvLiSIc' \
  --header 'Content-Type: application/json' \
  --data-raw '{"name": "avatars"}'
```

Then verify in Postgres (inside the CNPG primary pod):

```bash
psql -U db-owner -d stack-app -c 'select * from storage.buckets;'
```

You should see the `avatars` bucket row. The locale warnings from the container shell are harmless.

### Upload a file

Create a bucket and upload a file using the demo manifest (NodePort 30012) and the demo JWT:

```bash
# Create bucket (if not already present)
curl -X POST 'http://host.docker.internal:30012/bucket' \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoic2VydmljZV9yb2xlIiwiaWF0IjoxNjEzNTMxOTg1LCJleHAiOjE5MjkxMDc5ODV9.th84OKK0Iz8QchDyXZRrojmKSEZ-OuitQm_5DvLiSIc' \
  -H 'Content-Type: application/json' \
  -d '{"name": "avatars"}'

# Upload a small file
echo "hello storage" > hello.txt
curl -X POST 'http://host.docker.internal:30012/object/avatars/hello.txt' \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoic2VydmljZV9yb2xlIiwiaWF0IjoxNjEzNTMxOTg1LCJleHAiOjE5MjkxMDc5ODV9.th84OKK0Iz8QchDyXZRrojmKSEZ-OuitQm_5DvLiSIc' \
  -H 'Content-Type: text/plain' \
  --data-binary @hello.txt
```

Then check the DB:

```bash
psql -U db-owner -d stack-app -c 'select id, name, bucket_id, version from storage.objects;'
```
